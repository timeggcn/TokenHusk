//! TokenHusk 桌面端 — Tauri 2.0 入口。
//!
//! Phase 2：集成仪表盘、配置向导、系统托盘、安全兜底。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;

use tauri::Manager;

mod ipc;
mod tray;

/// 共享代理状态。
pub struct ProxyState {
    pub running: AtomicBool,
    pub paused: AtomicBool,
    pub started_at: Mutex<Option<Instant>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProxyState {
            running: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            started_at: Mutex::new(None),
        })
        .setup(|app| {
            // 启动本地代理（Phase 0 已验证，此处异步拉起）。
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let cfg = tokenhusk_core::proxy::server::ProxyConfig::from_env();
                if let Err(e) = tokenhusk_core::proxy::server::start(cfg).await {
                    tracing::error!(error = %e, "TokenHusk proxy exited");
                }
            });

            // 更新代理状态
            let state: tauri::State<'_, ProxyState> = app.state();
            state.running.store(true, std::sync::atomic::Ordering::Relaxed);
            *state.started_at.lock().unwrap() = Some(Instant::now());

            // 创建系统托盘
            if let Err(e) = tray::create_tray(app.handle()) {
                tracing::warn!(error = %e, "failed to create tray (non-fatal)");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::stats_commands::get_stats,
            ipc::stats_commands::get_recent_requests,
            ipc::stats_commands::get_request_detail,
            ipc::config_commands::detect_apps,
            ipc::config_commands::configure_app,
            ipc::config_commands::emergency_restore,
            ipc::config_commands::get_config_modification,
            ipc::quality_commands::submit_feedback,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
