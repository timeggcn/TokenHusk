//! 配置向导 IPC 命令（detect_apps / configure_app / emergency_restore）。
//!
//! 注意：所有命令不涉及 API Key 输入（方案 B 核心原则）。
//! 配置修改前创建 .bak 备份（AGENTS.md Rule 6）。

use tokenhusk_core::assistant::configurator::{self, auto_configure};
use tokenhusk_core::assistant::detector;
use tokenhusk_core::observation::models::{ConfigResult, DetectedApp};

/// 扫描已安装的 AI 应用。
#[tauri::command]
pub fn detect_apps() -> Vec<DetectedApp> {
    detector::detect_installed_apps()
}

/// 自动配置 A 级应用（修改 base_url，不修改 api_key）。
#[tauri::command]
pub fn configure_app(
    app_name: String,
    config_path: String,
    base_url_json_path: String,
    new_base_url: String,
    api_key_json_path: Option<String>,
) -> ConfigResult {
    auto_configure(&app_name, &config_path, &base_url_json_path, &new_base_url, api_key_json_path.as_deref())
}

/// 紧急还原：从所有 .bak 备份恢复原始配置。
#[tauri::command]
pub fn emergency_restore() -> Vec<ConfigResult> {
    configurator::emergency_restore()
}

/// 获取配置修改建议（供配置向导 Step 2 使用）。
#[tauri::command]
pub fn get_config_modification(app: DetectedApp) -> Option<serde_json::Value> {
    let modification = detector::get_config_modification(&app)?;
    Some(serde_json::to_value(modification).unwrap_or_default())
}
