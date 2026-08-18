//! tokenhusk.toml 配置模型（AGENTS.md §4.4）。
//!
//! 路由配置：`/v1/*` 路径前缀 → 上游 target；未命中 → `[proxy].upstream` 默认上游。
//! 解析失败 / 文件缺失都回落默认配置（Fail-Open 心态：配置缺失也能跑代理）。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use axum::http::HeaderMap;
use serde::Deserialize;

/// tokenhusk.toml 顶层配置。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    /// 路径前缀 → 上游 target（可带 match_header 条件）。
    #[serde(default)]
    pub routes: HashMap<String, RouteTarget>,
    #[serde(default)]
    pub proxy: ProxySettings,
}

/// 单条路由：把匹配的路径前缀转发到 target。
#[derive(Debug, Clone, Deserialize)]
pub struct RouteTarget {
    pub target: String,
    /// 可选：仅当请求头满足 `"Key: value"`（大小写不敏感）时才命中该路由。
    #[serde(default)]
    pub match_header: Option<String>,
}

/// `[proxy]` 段。
#[derive(Debug, Clone, Deserialize)]
pub struct ProxySettings {
    #[serde(default = "default_listen")]
    pub listen: SocketAddr,
    #[serde(default = "default_pipeline_timeout_ms")]
    pub pipeline_timeout_ms: u64,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    /// 默认上游：未命中 routes 时的 fallback target。
    #[serde(default = "default_upstream")]
    pub upstream: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            pipeline_timeout_ms: default_pipeline_timeout_ms(),
            default_mode: default_mode(),
            upstream: default_upstream(),
        }
    }
}

fn default_listen() -> SocketAddr {
    "127.0.0.1:10520".parse().expect("static address")
}
fn default_pipeline_timeout_ms() -> u64 {
    50
}
fn default_mode() -> String {
    "observe".to_string()
}
fn default_upstream() -> String {
    "https://api.openai.com".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config {0}: {1}")]
    Io(std::path::PathBuf, #[source] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[source] toml::de::Error),
}

impl AppConfig {
    /// 加载 tokenhusk.toml。调用方自行决定文件缺失/解析失败时的回落。
    pub fn load(path: &Path) -> Result<AppConfig, ConfigError> {
        let raw =
            std::fs::read_to_string(path).map_err(|e| ConfigError::Io(path.to_path_buf(), e))?;
        toml::from_str(&raw).map_err(ConfigError::Parse)
    }

    /// 解析某请求的上游 base URL：
    ///   1. 在 routes 里做「最长路径前缀」匹配；
    ///   2. 命中但要求 match_header 且不符 → 视作未命中；
    ///   3. 无命中 → 返回 `[proxy].upstream`（默认上游）。
    pub fn resolve_upstream(&self, path: &str, headers: &HeaderMap) -> String {
        let mut best: Option<(&str, &RouteTarget)> = None;
        for (route_path, target) in &self.routes {
            if !path.starts_with(route_path.as_str()) {
                continue;
            }
            if let Some(req) = &target.match_header {
                if !header_matches(headers, req) {
                    continue;
                }
            }
            let longer = match best {
                Some((bp, _)) => route_path.len() > bp.len(),
                None => true,
            };
            if longer {
                best = Some((route_path.as_str(), target));
            }
        }
        best.map_or_else(|| self.proxy.upstream.clone(), |(_, t)| t.target.clone())
    }
}

/// `"X-Provider: deepseek"` → `("x-provider", "deepseek")`，大小写不敏感比对。
fn header_matches(headers: &HeaderMap, spec: &str) -> bool {
    let (k, v) = match spec.split_once(':') {
        Some((k, v)) => (k.trim(), v.trim()),
        None => return false,
    };
    headers
        .get(k)
        .map(|val| val.to_str().map(|s| s.eq_ignore_ascii_case(v)).unwrap_or(false))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn default_upstream_is_openai() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.resolve_upstream("/v1/chat/completions", &HeaderMap::new()),
            "https://api.openai.com");
    }

    #[test]
    fn longest_prefix_route_wins() {
        let mut routes = HashMap::new();
        routes.insert(
            "/v1".to_string(),
            RouteTarget { target: "https://a.com".to_string(), match_header: None },
        );
        routes.insert(
            "/v1/messages".to_string(),
            RouteTarget { target: "https://b.com".to_string(), match_header: None },
        );
        let cfg = AppConfig { routes, proxy: ProxySettings::default() };
        assert_eq!(cfg.resolve_upstream("/v1/messages", &HeaderMap::new()), "https://b.com");
    }

    #[test]
    fn match_header_gates_route() {
        let mut routes = HashMap::new();
        routes.insert(
            "/v1/chat/completions".to_string(),
            RouteTarget {
                target: "https://deepseek.com".to_string(),
                match_header: Some("X-Provider: deepseek".to_string()),
            },
        );
        let cfg = AppConfig { routes, proxy: ProxySettings::default() };

        let mut with = HeaderMap::new();
        with.insert("x-provider", HeaderValue::from_static("deepseek"));
        assert_eq!(cfg.resolve_upstream("/v1/chat/completions", &with),
            "https://deepseek.com");

        let without = HeaderMap::new();
        assert_eq!(cfg.resolve_upstream("/v1/chat/completions", &without),
            "https://api.openai.com");
    }
}
