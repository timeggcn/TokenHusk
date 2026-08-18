// ─── IPC 数据模型 ─────────────────────────────────────────────
// 与 tokenhusk_core::observation::models 严格对齐。

export interface RequestRecord {
  id: number;
  timestamp: string;
  source_app: string;
  provider: string;
  model: string;
  original_input_tokens: number;
  compressed_input_tokens: number;
  output_tokens: number;
  saved_tokens: number;
  saved_ratio: number;
  estimated_cost_usd: number;
  saved_cost_usd: number;
  stages_applied: string[];
  compression_time_ms: number;
  skipped: boolean;
  skip_reason: string | null;
  message_count: number;
  has_code: boolean;
  has_json: boolean;
  has_log: boolean;
  original_body: string;
  compressed_body: string | null;
}

export interface DashboardOverview {
  today_requests: number;
  today_saved_tokens: number;
  today_saved_ratio: number;
  today_saved_cost: number;
  today_estimated_cost: number;
  total_requests_all_time: number;
  proxy_running: boolean;
  proxy_uptime_seconds: number;
  upstream: string;
}

export interface DetectedApp {
  name: string;
  config_path: string;
  level: string; // "A" | "B" | "C"
  configured: boolean;
  current_base_url: string | null;
  suggested_base_url: string;
  api_key_present: boolean;
  original_base_url: string | null;
}

export interface ConfigResult {
  app_name: string;
  success: boolean;
  backup_path: string | null;
  detail: string;
  test_connection_ok: boolean;
}

export interface Feedback {
  request_id: number;
  thumbs_up: boolean;
  comment: string | null;
  created_at: string;
}

// ─── 前端本地类型 ─────────────────────────────────────────────

export type TabType = "dashboard" | "logs" | "setup" | "safety";
export type ProxyStatus = "running" | "paused" | "stopped";

export interface TokenDistribution {
  name: string;
  value: number;
  color: string;
}

export interface SavingsSource {
  stage: string;
  saved_tokens: number;
  color: string;
}