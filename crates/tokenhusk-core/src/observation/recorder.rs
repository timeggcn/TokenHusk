//! SQLite 记录器（AGENTS.md §4.1 第 7 步）。
//!
//! 设计意图：
//! - 异步写入（`spawn_blocking`），不阻塞主转发流程
//! - 所有 Header 在入库前已脱敏（见 `proxy::headers::sanitize_headers`）
//! - 数据库路径：`$HOME/.tokenhusk/stats.db`（可通过 `TOKENHUSK_DB` 环境变量覆盖）

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::MutexGuard;

use rusqlite::Connection;
use tracing::warn;

use crate::observation::models::*;

/// 全局 Recorder 实例（进程内单例）。
static RECORDER: std::sync::LazyLock<Recorder> =
    std::sync::LazyLock::new(|| Recorder::new().expect("Recorder init"));

fn default_db_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".tokenhusk");
    std::fs::create_dir_all(&path).ok();
    path.push("stats.db");
    path
}

pub struct Recorder {
    conn: Mutex<Connection>,
}

impl Recorder {
    fn new() -> Result<Self, rusqlite::Error> {
        let db_path = std::env::var("TOKENHUSK_DB")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(default_db_path);
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS request_records (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp       TEXT NOT NULL,
                source_app      TEXT NOT NULL DEFAULT '',
                provider        TEXT NOT NULL DEFAULT '',
                model           TEXT NOT NULL DEFAULT '',
                original_input_tokens  INTEGER NOT NULL DEFAULT 0,
                compressed_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens   INTEGER NOT NULL DEFAULT 0,
                saved_tokens    INTEGER NOT NULL DEFAULT 0,
                saved_ratio     REAL NOT NULL DEFAULT 0.0,
                estimated_cost_usd  REAL NOT NULL DEFAULT 0.0,
                saved_cost_usd      REAL NOT NULL DEFAULT 0.0,
                stages_applied  TEXT NOT NULL DEFAULT '[]',
                compression_time_ms INTEGER NOT NULL DEFAULT 0,
                skipped         INTEGER NOT NULL DEFAULT 0,
                skip_reason     TEXT,
                message_count   INTEGER NOT NULL DEFAULT 0,
                has_code        INTEGER NOT NULL DEFAULT 0,
                has_json        INTEGER NOT NULL DEFAULT 0,
                has_log         INTEGER NOT NULL DEFAULT 0,
                original_body   TEXT NOT NULL DEFAULT '',
                compressed_body TEXT
            );
            CREATE TABLE IF NOT EXISTS backup_records (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name        TEXT NOT NULL,
                original_path   TEXT NOT NULL,
                backup_path     TEXT NOT NULL,
                created_at      TEXT NOT NULL,
                restored        INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS feedback (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id      INTEGER NOT NULL,
                thumbs_up       INTEGER NOT NULL DEFAULT 1,
                comment         TEXT,
                created_at      TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// 记录一次请求（异步，通过 spawn_blocking 写入）。
    pub fn record_request(record: RequestRecord) {
        let r = record;
        tokio::task::spawn_blocking(move || {
            let rec = &RECORDER;
            if let Ok(conn) = rec.conn.lock() {
                if let Err(e) = conn.execute(
                    "INSERT INTO request_records (
                        timestamp, source_app, provider, model,
                        original_input_tokens, compressed_input_tokens, output_tokens,
                        saved_tokens, saved_ratio,
                        estimated_cost_usd, saved_cost_usd,
                        stages_applied, compression_time_ms,
                        skipped, skip_reason,
                        message_count, has_code, has_json, has_log,
                        original_body, compressed_body
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
                    rusqlite::params![
                        r.timestamp, r.source_app, r.provider, r.model,
                        r.original_input_tokens, r.compressed_input_tokens, r.output_tokens,
                        r.saved_tokens, r.saved_ratio,
                        r.estimated_cost_usd, r.saved_cost_usd,
                        serde_json::to_string(&r.stages_applied).unwrap_or_default(), r.compression_time_ms,
                        r.skipped as i32, r.skip_reason,
                        r.message_count, r.has_code as i32, r.has_json as i32, r.has_log as i32,
                        r.original_body, r.compressed_body,
                    ],
                ) {
                    warn!(error = %e, "failed to record request");
                }
            }
        });
    }

    /// 查询今日概览。
    pub fn get_dashboard_overview() -> DashboardOverview {
        let rec = &RECORDER;
        let conn = match rec.conn.lock() {
            Ok(c) => c,
            Err(_) => return DashboardOverview::default(),
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let today_requests: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM request_records WHERE timestamp >= ?1",
                [&today],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let today_original: u64 = conn
            .query_row(
                "SELECT COALESCE(SUM(original_input_tokens), 0) FROM request_records WHERE timestamp >= ?1",
                [&today],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let today_compressed: u64 = conn
            .query_row(
                "SELECT COALESCE(SUM(compressed_input_tokens), 0) FROM request_records WHERE timestamp >= ?1",
                [&today],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let today_saved = today_original.saturating_sub(today_compressed);
        let today_ratio = if today_original > 0 {
            today_saved as f32 / today_original as f32
        } else {
            0.0
        };

        let today_cost: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(estimated_cost_usd), 0) FROM request_records WHERE timestamp >= ?1",
                [&today],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let today_saved_cost: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(saved_cost_usd), 0) FROM request_records WHERE timestamp >= ?1",
                [&today],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let total_all: u32 = conn
            .query_row("SELECT COUNT(*) FROM request_records", [], |row| row.get(0))
            .unwrap_or(0);

        DashboardOverview {
            today_requests,
            today_saved_tokens: today_saved,
            today_saved_ratio: today_ratio,
            today_saved_cost,
            today_estimated_cost: today_cost,
            total_requests_all_time: total_all,
            proxy_running: true,
            proxy_uptime_seconds: 0,
            upstream: std::env::var("TOKENHUSK_UPSTREAM").unwrap_or_else(|_| "https://api.openai.com".to_string()),
        }
    }

    /// 查询最近请求列表。
    pub fn get_recent_requests(limit: u32) -> Vec<RequestRecord> {
        let rec = &RECORDER;
        let conn = match rec.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, source_app, provider, model,
                    original_input_tokens, compressed_input_tokens, output_tokens,
                    saved_tokens, saved_ratio,
                    estimated_cost_usd, saved_cost_usd,
                    stages_applied, compression_time_ms,
                    skipped, skip_reason,
                    message_count, has_code, has_json, has_log,
                    original_body, compressed_body
                FROM request_records
                ORDER BY id DESC
                LIMIT ?1",
            )
            .unwrap();
        let rows = stmt.query_map([limit], |row| {
            let stages_json: String = row.get(12)?;
            Ok(RequestRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_app: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                original_input_tokens: row.get(5)?,
                compressed_input_tokens: row.get(6)?,
                output_tokens: row.get(7)?,
                saved_tokens: row.get(8)?,
                saved_ratio: row.get(9)?,
                estimated_cost_usd: row.get(10)?,
                saved_cost_usd: row.get(11)?,
                stages_applied: serde_json::from_str(&stages_json).unwrap_or_default(),
                compression_time_ms: row.get(13)?,
                skipped: row.get::<_, i32>(14)? != 0,
                skip_reason: row.get(15)?,
                message_count: row.get(16)?,
                has_code: row.get::<_, i32>(17)? != 0,
                has_json: row.get::<_, i32>(18)? != 0,
                has_log: row.get::<_, i32>(19)? != 0,
                original_body: row.get(20)?,
                compressed_body: row.get(21)?,
            })
        });
        rows.and_then(|r| r.collect::<Result<Vec<_>, _>>()).unwrap_or_default()
    }

    /// 查询单条请求详情。
    pub fn get_request_detail(request_id: u64) -> Option<RequestRecord> {
        let rec = &RECORDER;
        let conn = match rec.conn.lock() {
            Ok(c) => c,
            Err(_) => return None,
        };
        let mut stmt = conn
            .prepare("SELECT id, timestamp, source_app, provider, model,
                original_input_tokens, compressed_input_tokens, output_tokens,
                saved_tokens, saved_ratio,
                estimated_cost_usd, saved_cost_usd,
                stages_applied, compression_time_ms,
                skipped, skip_reason,
                message_count, has_code, has_json, has_log,
                original_body, compressed_body
            FROM request_records WHERE id = ?1")
            .ok()?;
        stmt.query_row([request_id], |row| {
            let stages_json: String = row.get(12)?;
            Ok(RequestRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_app: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                original_input_tokens: row.get(5)?,
                compressed_input_tokens: row.get(6)?,
                output_tokens: row.get(7)?,
                saved_tokens: row.get(8)?,
                saved_ratio: row.get(9)?,
                estimated_cost_usd: row.get(10)?,
                saved_cost_usd: row.get(11)?,
                stages_applied: serde_json::from_str(&stages_json).unwrap_or_default(),
                compression_time_ms: row.get(13)?,
                skipped: row.get::<_, i32>(14)? != 0,
                skip_reason: row.get(15)?,
                message_count: row.get(16)?,
                has_code: row.get::<_, i32>(17)? != 0,
                has_json: row.get::<_, i32>(18)? != 0,
                has_log: row.get::<_, i32>(19)? != 0,
                original_body: row.get(20)?,
                compressed_body: row.get(21)?,
            })
        }).ok()
    }

    /// 记录备份信息。
    pub fn record_backup(app_name: &str, original_path: &str, backup_path: &str) {
        let rec = &RECORDER;
        let app = app_name.to_string();
        let orig = original_path.to_string();
        let bak = backup_path.to_string();
        tokio::task::spawn_blocking(move || {
            if let Ok(conn) = rec.conn.lock() {
                let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                conn.execute(
                    "INSERT INTO backup_records (app_name, original_path, backup_path, created_at) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![app, orig, bak, now],
                ).ok();
            }
        });
    }

    /// 记录质量反馈。
    pub fn record_feedback(feedback: Feedback) {
        let f = feedback;
        tokio::task::spawn_blocking(move || {
            if let Ok(conn) = RECORDER.conn.lock() {
                conn.execute(
                    "INSERT INTO feedback (request_id, thumbs_up, comment, created_at) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![f.request_id, f.thumbs_up as i32, f.comment, f.created_at],
                ).ok();
            }
        });
    }

    /// 获取所有备份记录。
    pub fn get_backup_records() -> Vec<BackupRecord> {
        let rec = &RECORDER;
        let conn = match rec.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare("SELECT id, app_name, original_path, backup_path, created_at, restored FROM backup_records ORDER BY id DESC")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(BackupRecord {
                    id: row.get(0)?,
                    app_name: row.get(1)?,
                    original_path: row.get(2)?,
                    backup_path: row.get(3)?,
                    created_at: row.get(4)?,
                    restored: row.get::<_, i32>(5)? != 0,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    /// 标记备份为已还原。
    pub fn mark_backup_restored(backup_id: u64) {
        tokio::task::spawn_blocking(move || {
            if let Ok(conn) = RECORDER.conn.lock() {
                conn.execute(
                    "UPDATE backup_records SET restored = 1 WHERE id = ?1",
                    rusqlite::params![backup_id],
                )
                .ok();
            }
        });
    }

    /// 暴露内部连接（供 assistant::configurator 紧急还原时使用）。
    pub fn get_connection() -> Option<MutexGuard<'static, Connection>> {
        RECORDER.conn.lock().ok()
    }
}

impl Default for DashboardOverview {
    fn default() -> Self {
        Self {
            today_requests: 0,
            today_saved_tokens: 0,
            today_saved_ratio: 0.0,
            today_saved_cost: 0.0,
            today_estimated_cost: 0.0,
            total_requests_all_time: 0,
            proxy_running: false,
            proxy_uptime_seconds: 0,
            upstream: String::new(),
        }
    }
}
