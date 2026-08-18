//! TokenHusk 核心库。
//!
//! 设计意图：把「代理转发 / 压缩管线 / 计数 / 配置」放在一个纯 Rust crate 里，不依赖
//! Tauri/Tauri-Build。Phase 0 的 headless 验证（`cargo run`）不被桌面端构建链路（如
//! Windows 的 rc.exe）阻塞，Phase 2 再由 Tauri 壳拉起本 crate。

pub mod config;
pub mod counter;
pub mod pipeline;
pub mod proxy;
pub mod assistant;
pub mod observation;
