//! Header 处理：脱敏 + Key 泄露检测 + 逐跳头剥离。
//!
//! 与 AGENTS.md 的对齐：
//!   §5 / §8 —— 任何 Header 写入日志/DB 的路径必须经过 `sanitize_headers()`，
//!            该函数 `#[must_use]`，编译器强制调用方处理返回值，防裸头泄露。
//!   §6 —— 硬编码脱敏规则：Authorization → `Bearer [REDACTED]`。
//!   §7 —— `contains_key_leak()` 供 Key Leak Test 扫描日志/DB。

use axum::http::HeaderMap;

/// 任何将 Headers 写入日志/DB 的代码路径必须调用本函数（AGENTS.md §5 / §8）。
/// `#[must_use]` 强制编译器提醒调用方处理返回值，避免裸 headers 直接泄露进日志。
/// 实现严格遵循 AGENTS.md §6 的硬编码脱敏规则。
#[must_use]
pub fn sanitize_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(k, v)| {
            let key = k.as_str();
            let val = if key.eq_ignore_ascii_case("authorization") {
                "Bearer [REDACTED]".to_string()
            } else {
                v.to_str().unwrap_or("[binary]").to_string()
            };
            (key.to_string(), val)
        })
        .collect()
}

/// 检测文本是否包含疑似暴露的 API Key（AGENTS.md §7 Key Leak Test）。
/// 保守匹配：`sk-` 后跟 ≥ 20 个字母/数字字符。
pub fn contains_key_leak(text: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0;
    while i + 3 <= b.len() {
        if &b[i..i + 3] == b"sk-" {
            let mut n = 0;
            let mut j = i + 3;
            while j < b.len() && b[j].is_ascii_alphanumeric() {
                n += 1;
                j += 1;
            }
            if n >= 20 {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// 剥离逐跳（hop-by-hop）头。Authorization / Accept / Content-Type / X-* 等原样保留。
/// 理由见 stream.rs 的坑点 4。
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "host",
    "content-length",
    "transfer-encoding",
    "keep-alive",
    "te",
    "trailer",
    "upgrade",
    "proxy-authenticate",
    "proxy-authorization",
];

pub fn strip_hop_by_hop(headers: &HeaderMap) -> HeaderMap {
    headers
        .iter()
        .filter(|(name, _)| !HOP_BY_HOP.iter().any(|h| name.as_str().eq_ignore_ascii_case(h)))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn sanitize_redacts_authorization_no_key_leak() {
        let mut h = HeaderMap::new();
        h.insert(
            "authorization",
            HeaderValue::from_static("Bearer sk-ABCDEFGHIJKLMNOPQRSTUVWXY"),
        );
        h.insert("x-request-id", HeaderValue::from_static("req-1"));
        let out = sanitize_headers(&h);
        let auth = out
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .unwrap()
            .1
            .clone();
        assert_eq!(auth, "Bearer [REDACTED]");
        assert!(
            !contains_key_leak(&format!("{out:?}")),
            "serialized sanitized headers leaked"
        );
        assert!(out
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("x-request-id") && v == "req-1"));
    }

    #[test]
    fn contains_key_leak_detects_and_rejects() {
        assert!(contains_key_leak("Bearer sk-ABCDEFGHIJKLMNOPQRSTUVWXY"));
        assert!(contains_key_leak("leak sk-ABCDEFGHIJKLMNOPQRSTUVWXY end"));
        assert!(!contains_key_leak("Bearer [REDACTED]"));
        assert!(!contains_key_leak("sk-tooshort"));
        assert!(!contains_key_leak("asking-about-sk-keys"));
    }

    #[test]
    fn strips_hop_by_hop_but_keeps_auth() {
        let mut h = HeaderMap::new();
        h.insert("authorization", HeaderValue::from_static("Bearer sk-test"));
        h.insert("content-length", HeaderValue::from_static("123"));
        h.insert("connection", HeaderValue::from_static("keep-alive"));
        h.insert("x-request-id", HeaderValue::from_static("req-1"));
        let out = strip_hop_by_hop(&h);
        assert!(out.contains_key("authorization"));
        assert!(out.contains_key("x-request-id"));
        assert!(!out.contains_key("content-length"));
        assert!(!out.contains_key("connection"));
    }
}
