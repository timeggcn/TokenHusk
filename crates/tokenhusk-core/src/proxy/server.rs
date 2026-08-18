//! 本地代理 HTTP 服务器（axum）。
//!
//! 路由规则：
//!   GET  /health       -> 200 {"status":"ok",...}
//!   ANY  /v1/*         -> 流式转发到上游（来自 tokenhusk.toml 路由表 / TOKENHUSK_UPSTREAM）
//!   其余               -> 404
//!
//! 与 AGENTS.md Golden Rules 对齐：
//!   Rule 1 Fail-Open / Rule 2 50ms 预算约束【压缩管线】；Phase 0 透传不接压缩，天然满足。
//!   Rule 4 零密钥存储：Authorization 仅随请求流经内存、原样转发，绝不落盘/打印。

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use axum::routing::{any, get};
use axum::Router;
use tracing::info;

use crate::config::AppConfig;
use crate::proxy::stream::{forward_request, health, AppState};

pub const DEFAULT_BIND: &str = "127.0.0.1:10520";

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to bind {0}")]
    Bind(std::io::Error),
    #[error("server error: {0}")]
    Serve(std::io::Error),
}

/// 代理启动配置：绑定地址 + 路由表 + 上游环境变量覆盖（测试用）。
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub bind: SocketAddr,
    pub app_config: AppConfig,
    pub upstream_env_override: Option<String>,
}

impl ProxyConfig {
    /// 从环境变量构建：
    ///   TOKENHUSK_BIND     -> 监听地址（默认 127.0.0.1:10520）
    ///   TOKENHUSK_UPSTREAM -> 全局上游覆盖（供 mock 端到端测试）
    ///   tokenhusk.toml     -> /v1/* 路径 → target 路由表（AGENTS.md §4.4）
    pub fn from_env() -> Self {
        let bind: SocketAddr = std::env::var("TOKENHUSK_BIND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| DEFAULT_BIND.parse().expect("DEFAULT_BIND valid"));
        // tokenhusk.toml 缺失或解析失败都回落默认（Fail-Open 心态：配置缺失也能跑）。
        let app_config = AppConfig::load(Path::new("tokenhusk.toml")).unwrap_or_default();
        let upstream_env_override = std::env::var("TOKENHUSK_UPSTREAM").ok();
        Self {
            bind,
            app_config,
            upstream_env_override,
        }
    }
}

/// 上游解析器：env 覆盖优先，否则查 tokenhusk.toml 路由表。
pub struct UpstreamResolver {
    app_config: AppConfig,
    env_override: Option<String>,
}

impl UpstreamResolver {
    pub fn new(app_config: AppConfig, env_override: Option<String>) -> Self {
        Self {
            app_config,
            env_override,
        }
    }

    pub fn resolve(&self, path: &str, headers: &axum::http::HeaderMap) -> String {
        if let Some(override_upstream) = &self.env_override {
            return override_upstream.clone();
        }
        self.app_config.resolve_upstream(path, headers)
    }
}

/// 构建路由（供测试注入自定义上游 / 默认上游）。
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        // axum 0.7 通配路由语法：*path 匹配任意深度后缀。
        .route("/v1/*path", any(forward_request))
        .fallback(|| async { (axum::http::StatusCode::NOT_FOUND, "not found") })
        .with_state(state)
}

/// 启动代理服务器，阻塞直到收到 Ctrl-C（优雅关闭）。
pub async fn start(cfg: ProxyConfig) -> Result<(), ServerError> {
    // 只设 connect_timeout，绝不设整体 timeout（坑点 2，见 stream.rs）。
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        // 禁用系统代理：Windows 上系统代理可能拦截 loopback 连接（见集成测试）。
        .no_proxy()
        // 不启用 gzip/brotli（坑点 3）。
        .build()
        .expect("reqwest client build only fails on invalid TLS config");

    let state = Arc::new(AppState::new(
        client,
        UpstreamResolver::new(cfg.app_config.clone(), cfg.upstream_env_override),
    ));

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(cfg.bind)
        .await
        .map_err(ServerError::Bind)?;
    info!(bind = %cfg.bind, "TokenHusk proxy listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Serve)
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("shutdown signal received, draining in-flight connections");
}
