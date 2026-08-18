//! Phase 0 headless 验证入口（`cargo run` 默认启动这里，而非 Tauri GUI）。
//!
//! 为什么独立 bin：Tauri GUI 的 `cargo run` 会拉起窗口并依赖前端 devServer，
//! 而验收要求 headless 启动后 curl /health。GUI 用 `cargo run --bin tokenhusk`（Phase 2）。

use tokenhusk_core::counter::token_counter;
use tokenhusk_core::pipeline::structure::json_crusher;
use tokenhusk_core::proxy::server::{self, ProxyConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    print_banner();
    report_json_crusher();
    report_token_counter();
    print_usage();

    let cfg = ProxyConfig::from_env();
    server::start(cfg).await?;

    Ok(())
}

fn print_banner() {
    println!("======================================================");
    println!("  TokenHusk · Phase 0 技术验证 (headless)");
    println!("======================================================\n");
}

/// PoC 1：JSON 压缩 —— 展示一个真实的「巨大工具输出」压缩前后对比。
fn report_json_crusher() {
    use serde_json::json;
    // 模拟 Agent 场景：50 个用户、每项带 meta:null + 调试字段。
    let users: Vec<serde_json::Value> = (0..50)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("user_{i}"),
                "email": format!("user_{i}@example.com"),
                "meta": null,
            })
        })
        .collect();
    let input = json!({
        "status": "success",
        "data": {
            "users": users,
            "pagination": { "page": 1, "total": 50, "per_page": 50 },
            "debug": {},
            "trace_id": "abc-123-def",
        }
    });

    let before = serde_json::to_string(&input).expect("serialize");
    let after = serde_json::to_string(&json_crusher::json_crusher_poc(&input)).expect("serialize");
    let ratio = 1.0 - (after.len() as f64) / (before.len() as f64);

    println!("[1/2] JSON Crusher PoC");
    println!("  before : {} bytes (compact)", before.len());
    println!("  after  : {} bytes (compact)", after.len());
    println!("  ratio  : {:.1}% (目标 > 30%)", ratio * 100.0);
    println!("  preview: {}\n", &after.chars().take(200).collect::<String>());
}

/// PoC 2：Token 计数 —— 同一段文本分别用 cl100k 与 o200k 编码。
fn report_token_counter() {
    let text = token_counter::sample_text();
    let counts = token_counter::count_tokens(&text);
    println!("[2/2] Token Counter PoC (cl100k vs o200k)");
    println!("  text     : {}", &text.chars().take(80).collect::<String>());
    println!("  cl100k   : {} tokens", counts.cl100k);
    println!("  o200k    : {} tokens", counts.o200k);
    println!("  delta    : {:+.1}%\n", counts.delta_pct());
}

fn print_usage() {
    println!("── 启动代理 ──────────────────────────────────────");
    println!("  健康检查 : curl http://127.0.0.1:10520/health");
    println!("  端到端   : bash scripts/test_sse.sh   (mock 上游)");
    println!("  转发上游 : TOKENHUSK_UPSTREAM=<url> 覆盖默认 https://api.openai.com");
    println!("  Ctrl-C 优雅关闭\n");
}
