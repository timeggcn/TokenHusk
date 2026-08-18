//! 统计相关 IPC 命令（get_stats / get_recent_requests / get_request_detail）。
//!
//! 注意：所有返回值不得包含 Authorization 信息（AGENTS.md Rule 4）。

use tauri::State;

use tokenhusk_core::observation::models::{DashboardOverview, RequestRecord};
use tokenhusk_core::observation::recorder::Recorder;

/// 获取今日仪表盘概览。
#[tauri::command]
pub fn get_stats(proxy_running: State<'_, crate::ProxyState>) -> DashboardOverview {
    let mut overview = Recorder::get_dashboard_overview();
    overview.proxy_running = proxy_running.running.load(std::sync::atomic::Ordering::Relaxed);
    overview.proxy_uptime_seconds = proxy_running
        .started_at
        .lock()
        .unwrap()
        .map(|t| {
            let elapsed = std::time::Instant::now().duration_since(t);
            elapsed.as_secs()
        })
        .unwrap_or(0);
    overview
}

/// 获取最近请求列表（默认 50 条）。
#[tauri::command]
pub fn get_recent_requests(limit: Option<u32>) -> Vec<RequestRecord> {
    Recorder::get_recent_requests(limit.unwrap_or(50))
}

/// 获取单条请求详情（含 Diff 所需原始/压缩 body）。
#[tauri::command]
pub fn get_request_detail(request_id: u64) -> Option<RequestRecord> {
    Recorder::get_request_detail(request_id)
}
