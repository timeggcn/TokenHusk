//! 配置备份/修改/还原（AGENTS.md §6 配置助手 + §7 紧急还原）。
//!
//! 设计意图：
//!   - 修改前必须创建 .bak 备份文件（AGENTS.md Rule 6）
//!   - 只修改 base_url，不修改 api_key 字段（方案 B 核心原则）
//!   - 紧急还原不依赖代理运行状态（直接从 .bak 恢复）

use std::path::Path;

use serde_json::Value;

use crate::observation::models::ConfigResult;
use crate::observation::recorder::Recorder;

/// 自动修改 A 级应用的配置文件（ChatBox / Cherry Studio）。
///
/// 操作顺序：
///   1. 读取原文件 + 解析 JSON
///   2. 备份原文件 → `{path}.bak.{timestamp}`
///   3. 修改 base_url（不修改 api_key）
///   4. 写回文件
///   5. 记录备份到 SQLite
///   6. 发送测试请求验证连通性
pub fn auto_configure(
    app_name: &str,
    config_path: &str,
    base_url_json_path: &str,
    new_base_url: &str,
    _api_key_json_path: Option<&str>,
) -> ConfigResult {
    let path = Path::new(config_path);
    if !path.exists() {
        return ConfigResult {
            app_name: app_name.to_string(),
            success: false,
            backup_path: None,
            detail: "配置文件不存在".to_string(),
            test_connection_ok: false,
        };
    }

    // 1. 读取原文件
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return ConfigResult {
                app_name: app_name.to_string(),
                success: false,
                backup_path: None,
                detail: format!("读取失败: {e}"),
                test_connection_ok: false,
            };
        }
    };

    let mut json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return ConfigResult {
                app_name: app_name.to_string(),
                success: false,
                backup_path: None,
                detail: format!("JSON 解析失败: {e}"),
                test_connection_ok: false,
            };
        }
    };

    // 2. 备份原文件
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = format!("{config_path}.bak.{timestamp}");
    if let Err(e) = std::fs::copy(path, &backup_path) {
        return ConfigResult {
            app_name: app_name.to_string(),
            success: false,
            backup_path: None,
            detail: format!("备份失败: {e}"),
            test_connection_ok: false,
        };
    }

    // 3. 修改 base_url（JSON 路径如 "openaiBaseUrl" 或 "api.baseUrl"）
    let keys: Vec<&str> = base_url_json_path.split('.').collect();
    set_json_value(&mut json, &keys, Value::String(new_base_url.to_string()));

    // 不修改 api_key 字段（方案 B 核心原则）
    // 如果 api_key 字段存在，原样保留

    // 4. 写回文件
    let new_content = match serde_json::to_string_pretty(&json) {
        Ok(c) => c,
        Err(e) => {
            return ConfigResult {
                app_name: app_name.to_string(),
                success: false,
                backup_path: Some(backup_path),
                detail: format!("序列化失败: {e}"),
                test_connection_ok: false,
            };
        }
    };
    if let Err(e) = std::fs::write(path, &new_content) {
        return ConfigResult {
            app_name: app_name.to_string(),
            success: false,
            backup_path: Some(backup_path),
            detail: format!("写入失败: {e}"),
            test_connection_ok: false,
        };
    }

    // 5. 记录备份到 SQLite
    Recorder::record_backup(app_name, config_path, &backup_path);

    // 6. 发送测试请求验证连通性
    let test_ok = test_connection(new_base_url).unwrap_or(false);

    ConfigResult {
        app_name: app_name.to_string(),
        success: true,
        backup_path: Some(backup_path.clone()),
        detail: if test_ok {
            format!("配置成功！已备份到 {backup_path}，连接测试通过")
        } else {
            format!("配置成功！已备份到 {backup_path}，但连接测试失败（代理可能未运行）")
        },
        test_connection_ok: test_ok,
    }
}

/// 紧急还原：从所有 .bak 文件恢复原始配置。
///
/// 不依赖代理运行状态。还原后更新 SQLite 中的备份记录。
pub fn emergency_restore() -> Vec<ConfigResult> {
    let records = Recorder::get_backup_records();
    let mut results = Vec::new();
    for record in &records {
        if record.restored {
            continue;
        }
        let bak_path = Path::new(&record.backup_path);
        let orig_path = Path::new(&record.original_path);
        if !bak_path.exists() {
            results.push(ConfigResult {
                app_name: record.app_name.clone(),
                success: false,
                backup_path: Some(record.backup_path.clone()),
                detail: "备份文件不存在".to_string(),
                test_connection_ok: false,
            });
            continue;
        }
        match std::fs::copy(bak_path, orig_path) {
            Ok(_) => {
                // 标记为已还原
                Recorder::mark_backup_restored(record.id);
                results.push(ConfigResult {
                    app_name: record.app_name.clone(),
                    success: true,
                    backup_path: Some(record.backup_path.clone()),
                    detail: format!("已从 {} 还原", record.backup_path),
                    test_connection_ok: false,
                });
            }
            Err(e) => {
                results.push(ConfigResult {
                    app_name: record.app_name.clone(),
                    success: false,
                    backup_path: Some(record.backup_path.clone()),
                    detail: format!("还原失败: {e}"),
                    test_connection_ok: false,
                });
            }
        }
    }
    results
}

/// 在 JSON 中按路径设置值（如 ["api", "baseUrl"] → json["api"]["baseUrl"]）。
fn set_json_value(json: &mut Value, keys: &[&str], value: Value) {
    if keys.is_empty() {
        return;
    }
    if keys.len() == 1 {
        if let Value::Object(ref mut map) = json {
            map.insert(keys[0].to_string(), value);
        }
        return;
    }
    if let Value::Object(ref mut map) = json {
        if let Some(inner) = map.get_mut(keys[0]) {
            set_json_value(inner, &keys[1..], value);
        }
    }
}

/// 发送测试请求验证代理连通性。
fn test_connection(upstream: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .no_proxy()
            .build()?;
        let url = format!("{upstream}/health");
        let resp = client.get(&url).send().await?;
        Ok(resp.status().is_success())
    })
}

/// 暴露一个 pub 函数供 src-tauri 的 IPC 命令调用（获取 SQLite 连接）。
/// 用于紧急还原时更新备份记录。
pub fn get_recorder_connection() -> Option<std::sync::MutexGuard<'static, rusqlite::Connection>> {
    Recorder::get_connection()
}
