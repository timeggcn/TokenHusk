//! 请求转发 + SSE 流式透传（核心）。
//!
//! 【axum + reqwest SSE 透传的已知坑，均在此标注并处理】
//!
//! 1. 【绝不缓冲响应体】不要用 `resp.text().await` / `resp.bytes().await`。
//!    那样会把整个 SSE 流缓冲进内存，客户端在上游结束前收不到一个字节。
//!    正确：`resp.bytes_stream()` → `Body::from_stream` 逐块写。
//! 2. 【不设整体超时】reqwest `.timeout()` 作用于「整个响应体接收完成」，会把分钟级
//!    SSE 流掐断。只设 connect_timeout（server.rs 客户端构建处）。Rule 2 的 50ms 预算
//!    只约束压缩管线，转发链路本身允许长时间。
//! 3. 【不自动解压】不启用 reqwest 的 gzip/brotli feature，否则会透明解压响应体却仍透传
//!    原始 Content-Encoding 头 → 客户端对解压字节二次解压 → 乱码。
//! 4. 【剥离 hop-by-hop 头】connection/host/content-length/transfer-encoding 等必须剥离：
//!    host 由 reqwest 按上游重生成；content-length 重新分块后失真（见 headers.rs）。
//! 5. 【请求体也流式】上行 body 经 `Body::into_data_stream()` → `reqwest::Body::wrap_stream`，
//!    不缓冲。
//! 6. 【上游失败显式返回 502】Rule 1（Fail-Open）约束的是「压缩失败回退原始请求」，
//!    不是「转发失败也吞掉」。上游连不上必须 502 显式暴露，不能返回 500 或挂起。

use std::sync::Arc;

use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;
use tracing::warn;

use crate::proxy::headers::{sanitize_headers, strip_hop_by_hop};
use crate::proxy::server::UpstreamResolver;

pub struct AppState {
    pub client: reqwest::Client,
    pub resolver: UpstreamResolver,
}

impl AppState {
    pub fn new(client: reqwest::Client, resolver: UpstreamResolver) -> Self {
        Self { client, resolver }
    }
}

/// GET /health：200 + JSON，附带 CORS 便于 Tauri webview / 浏览器直接 fetch。
pub async fn health() -> Response {
    let mut res = Json(json!({ "status": "ok", "service": "tokenhusk-proxy" })).into_response();
    res.headers_mut().insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    res
}

/// 透传 `/v1/*` 到上游，SSE 逐块转发，不缓冲。
pub async fn forward_request(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    OriginalUri(uri): OriginalUri,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Response {
    // 1. 决定上游：env 覆盖 > routes 前缀匹配 > 默认上游。
    let upstream = state.resolver.resolve(uri.path(), &headers);
    let path_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let url = format!("{upstream}{path_query}");

    let req = state
        .client
        .request(method, &url)
        .headers(strip_hop_by_hop(&headers))
        .body(reqwest::Body::wrap_stream(body.into_data_stream()));

    let upstream_resp = match req.send().await {
        Ok(r) => r,
        // 坑点 6：上游失败显式 502，且只在日志里写「脱敏后」的 headers（AGENTS.md §5）。
        Err(e) => {
            warn!(
                error = %e, url = %url,
                sanitized_headers = ?sanitize_headers(&headers),
                "upstream connect/send failed"
            );
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "upstream unreachable", "detail": e.to_string() })),
            )
                .into_response();
        }
    };

    // 2. 下游响应：状态码 + 过滤后的头原样透传，SSE 逐块转发（坑点 1/4/5）。
    let status = StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let out_headers = strip_hop_by_hop(upstream_resp.headers());

    // 关键：bytes_stream → from_stream，逐块转发，绝不 await 整个 body。
    let body = Body::from_stream(upstream_resp.bytes_stream());

    let mut res = Response::builder().status(status).body(body).expect("valid status/body");
    *res.headers_mut() = out_headers;
    res
}

// ───────────────────── 集成测试：Header 透传完整性（byte-level） ─────────────────────
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use axum::http::Request;
    use tokio::sync::Mutex;
    use tower::util::ServiceExt;

    async fn spawn_echo_backend(captured: Arc<Mutex<Option<HeaderMap>>>) -> SocketAddr {
        let inner = captured.clone();
        let backend = axum::Router::new().fallback(
            move |headers: HeaderMap, _body: Body| async move {
                *inner.lock().await = Some(headers);
                (StatusCode::OK, "ok")
            },
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, backend).await.unwrap();
        });
        // 给后端服务器一点时间开始 accept。
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        addr
    }

    #[tokio::test]
    async fn authorization_and_custom_headers_passthrough_byte_level() {
        let captured = Arc::new(Mutex::new(None));
        let backend_addr = spawn_echo_backend(captured.clone()).await;

        // 先用 reqwest 直接验证后端可达。
        let probe = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap();
        let probe_result = probe
            .post(&format!("http://{backend_addr}/ping"))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;
        if let Err(e) = &probe_result {
            panic!("backend not reachable: {e}");
        }

        let resolver = UpstreamResolver::new(
            crate::config::AppConfig::default(),
            Some(format!("http://{backend_addr}")),
        );
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap();
        let state = Arc::new(AppState::new(client, resolver));
        let proxy = crate::proxy::server::build_router(state);

        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("authorization", "Bearer sk-ABCDEFGHIJKLMNOPQRSTUVWXY")
            .header("x-request-id", "req-byte-level-001")
            .header("x-custom-tokenhusk", "should-survive")
            .body(Body::from("{}"))
            .unwrap();

        let resp = proxy.oneshot(req).await.unwrap();
        let status = resp.status();
        if status != StatusCode::OK {
            let body = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
            panic!("proxy returned {status}: {}", String::from_utf8_lossy(&body));
        }

        let got = captured.lock().await.take().expect("backend received request");
        let auth = got
            .get("authorization")
            .expect("authorization arrived")
            .to_str()
            .unwrap();
        assert_eq!(
            auth, "Bearer sk-ABCDEFGHIJKLMNOPQRSTUVWXY",
            "Authorization byte-level mismatch"
        );
        assert_eq!(
            got.get("x-request-id").unwrap().to_str().unwrap(),
            "req-byte-level-001"
        );
        assert_eq!(
            got.get("x-custom-tokenhusk").unwrap().to_str().unwrap(),
            "should-survive"
        );
        // 逐跳头不应出现在后端
        assert!(got.get("content-length").is_none(), "content-length must be stripped");
    }
}
