//! 观测层数据模型（AGENTS.md §4.1）。
//!
//! 设计意图：与 plan.md 的 RequestRecord / DailyStats 定义严格对齐。
//! 所有 IPC 序列化途经此模块，确保 Authorization 等敏感字段不暴露。

use serde::{Deserialize, Serialize};

/// 每次 API 请求的完整记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub id: u64,
    pub timestamp: String, // ISO 8601

    // 来源
    pub source_app: String,
    pub provider: String,
    pub model: String,

    // Token 统计
    pub original_input_tokens: u32,
    pub compressed_input_tokens: u32,
    pub output_tokens: u32,
    pub saved_tokens: i32, // original - compressed（可能为负）
    pub saved_ratio: f32,

    // 费用估算
    pub estimated_cost_usd: f64,
    pub saved_cost_usd: f64,

    // 压缩详情
    pub stages_applied: Vec<String>,
    pub compression_time_ms: u32,
    pub skipped: bool,
    pub skip_reason: Option<String>,

    // 元数据
    pub message_count: u32,
    pub has_code: bool,
    pub has_json: bool,
    pub has_log: bool,

    // 请求详情（用于 Diff 查看，不包含 Authorization）
    pub original_body: String,
    pub compressed_body: Option<String>,
}

/// 聚合统计（单日）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub total_requests: u32,
    pub total_original_tokens: u64,
    pub total_compressed_tokens: u64,
    pub total_saved_tokens: i64,
    pub total_estimated_cost: f64,
    pub total_saved_cost: f64,
    pub by_app: std::collections::HashMap<String, AppStats>,
    pub by_provider: std::collections::HashMap<String, ProviderStats>,
    pub by_stage: std::collections::HashMap<String, StageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStats {
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub saved_tokens: i64,
    pub requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub saved_tokens: i64,
    pub requests: u32,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageStats {
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub saved_tokens: i64,
    pub requests: u32,
}

/// 仪表盘概览（一次性返回今日关键指标）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub today_requests: u32,
    pub today_saved_tokens: u64,
    pub today_saved_ratio: f32,
    pub today_saved_cost: f64,
    pub today_estimated_cost: f64,
    pub total_requests_all_time: u32,
    pub proxy_running: bool,
    pub proxy_uptime_seconds: u64,
    pub upstream: String,
}

/// 配置备份记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: u64,
    pub app_name: String,
    pub original_path: String,
    pub backup_path: String,
    pub created_at: String,
    pub restored: bool,
}

/// 检测到的已安装 AI 应用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedApp {
    pub name: String,
    pub config_path: String,
    pub level: String, // "A" | "B" | "C"
    pub configured: bool,
    pub current_base_url: Option<String>,
    pub suggested_base_url: String,
    pub api_key_present: bool,
    pub original_base_url: Option<String>,
    /// 配置文件中 base_url 的 JSON 路径（如 "openaiBaseUrl"）。
    pub base_url_json_path: Option<String>,
    /// 配置文件中 api_key 的 JSON 路径（如 "openaiKey"）。
    pub api_key_json_path: Option<String>, // 备份的原始值
}

/// 配置执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResult {
    pub app_name: String,
    pub success: bool,
    pub backup_path: Option<String>,
    pub detail: String,
    pub test_connection_ok: bool,
}

/// 质量反馈。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub request_id: u64,
    pub thumbs_up: bool,
    pub comment: Option<String>,
    pub created_at: String,
}
