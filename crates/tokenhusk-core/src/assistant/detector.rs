//! 已安装 AI 应用检测（配置助手 Layer 0）。
//!
//! 设计意图：扫描常见安装路径/环境变量/进程，检测用户已安装的 AI 应用。
//! 安全分级（AGENTS.md §6.1）：
//!   A 级：ChatBox、Cherry Studio（自动修改）
//!   B 级：Cursor（半自动，显示 Diff 确认）
//!   C 级：Claude Code（仅提供教程）

use std::path::PathBuf;

use serde_json::Value;

use crate::observation::models::DetectedApp;

/// 常用 AI 应用的配置路径与检测规则。
struct AppRule {
    pub name: &'static str,
    pub level: &'static str,
    pub config_path: fn() -> PathBuf,
    pub base_url_key: &'static str,
    pub api_key_key: Option<&'static str>,
    /// 配置文件内 base_url 的 JSON 路径（如 "api.baseUrl"）。
    pub base_url_json_path: &'static str,
    pub api_key_json_path: Option<&'static str>,
    pub suggested_base_url: &'static str,
}

static APP_RULES: &[AppRule] = &[
    // ── A 级 ──────────────────────────────────────────────────────
    AppRule {
        name: "ChatBox",
        level: "A",
        config_path: || {
            let mut p = dirs::home_dir().unwrap_or_default();
            p.push(".config/chatbox/config.json");
            p
        },
        base_url_key: "openaiBaseUrl",
        api_key_key: Some("openaiKey"),
        base_url_json_path: "openaiBaseUrl",
        api_key_json_path: Some("openaiKey"),
        suggested_base_url: "http://127.0.0.1:10520",
    },
    AppRule {
        name: "Cherry Studio",
        level: "A",
        config_path: || {
            let mut p = dirs::home_dir().unwrap_or_default();
            p.push(".config/cherry-studio/settings.json");
            p
        },
        base_url_key: "api.baseUrl",
        api_key_key: Some("api.key"),
        base_url_json_path: "api.baseUrl",
        api_key_json_path: Some("api.key"),
        suggested_base_url: "http://127.0.0.1:10520",
    },
    // ── B 级 ──────────────────────────────────────────────────────
    AppRule {
        name: "Cursor",
        level: "B",
        config_path: || {
            let mut p = dirs::home_dir().unwrap_or_default();
            p.push(".cursor/settings.json");
            p
        },
        base_url_key: "openAiBaseUrl",
        api_key_key: Some("openAiApiKey"),
        base_url_json_path: "openAiBaseUrl",
        api_key_json_path: Some("openAiApiKey"),
        suggested_base_url: "http://127.0.0.1:10520",
    },
    // ── C 级 ──────────────────────────────────────────────────────
    AppRule {
        name: "Claude Code",
        level: "C",
        config_path: || {
            let mut p = dirs::home_dir().unwrap_or_default();
            p.push(".claude/settings.json");
            p
        },
        base_url_key: "apiBaseUrl",
        api_key_key: None,
        base_url_json_path: "apiBaseUrl",
        api_key_json_path: None,
        suggested_base_url: "http://127.0.0.1:10520",
    },
];

/// 扫描所有已知 AI 应用，返回检测结果。
pub fn detect_installed_apps() -> Vec<DetectedApp> {
    let mut results = Vec::new();
    for rule in APP_RULES {
        let config_path = (rule.config_path)();
        let exists = config_path.exists();
        let (current_base_url, api_key_present, original_base_url) = if exists {
            read_config_fields(&config_path, rule)
        } else {
            (None, false, None)
        };
        let configured = current_base_url
            .as_deref()
            .map(|u| u.contains("127.0.0.1:10520") || u.contains("localhost:10520"))
            .unwrap_or(false);
        results.push(DetectedApp {
            name: rule.name.to_string(),
            config_path: config_path.to_string_lossy().to_string(),
            level: rule.level.to_string(),
            configured,
            current_base_url,
            suggested_base_url: rule.suggested_base_url.to_string(),
            api_key_present,
            original_base_url,
            base_url_json_path: Some(rule.base_url_json_path.to_string()),
            api_key_json_path: rule.api_key_json_path.map(|s| s.to_string()),
        });
    }
    results
}

/// 读取配置文件的指定字段。
fn read_config_fields(path: &PathBuf, rule: &AppRule) -> (Option<String>, bool, Option<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (None, false, None),
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, false, None),
    };

    let base_url = json
        .pointer(&format!("/{}", rule.base_url_json_path.replace('.', "/")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let api_key_present = rule
        .api_key_json_path
        .and_then(|path| {
            json.pointer(&format!("/{}", path.replace('.', "/")))
                .and_then(|v| v.as_str())
        })
        .map(|s| !s.is_empty() && s != "sk-")
        .unwrap_or(false);

    (base_url, api_key_present, None)
}

/// 为指定应用生成配置修改指令（供配置向导使用）。
pub fn get_config_modification(app: &DetectedApp) -> Option<ConfigModification> {
    if app.level == "C" {
        return Some(ConfigModification {
            app_name: app.name.clone(),
            action: ConfigAction::ShowTutorial {
                steps: vec![
                    format!("1. 打开 {} 设置", app.name),
                    format!("2. 将 API 地址改为：{}", app.suggested_base_url),
                    "3. API Key 保持不变".to_string(),
                    "4. 保存设置并测试连接".to_string(),
                ],
                copy_text: app.suggested_base_url.clone(),
            },
        });
    }
    if app.level == "B" {
        return Some(ConfigModification {
            app_name: app.name.clone(),
            action: ConfigAction::ShowDiff {
                config_path: app.config_path.clone(),
                original_base_url: app.current_base_url.clone().unwrap_or_default(),
                new_base_url: app.suggested_base_url.clone(),
                diff_lines: vec![
                    format!("- \"{}\": \"{}\"", app.base_url_json_path.as_deref().unwrap_or(""), app.current_base_url.as_deref().unwrap_or("(none)")),
                    format!("+ \"{}\": \"{}\"", app.base_url_json_path.as_deref().unwrap_or(""), app.suggested_base_url),
                ],
            },
        });
    }
    // A 级：直接提供修改数据
    Some(ConfigModification {
        app_name: app.name.clone(),
        action: ConfigAction::AutoModify {
            config_path: app.config_path.clone(),
            base_url_json_path: app.base_url_json_path.clone().unwrap_or_default(),
            new_base_url: app.suggested_base_url.clone(),
            api_key_json_path: app.api_key_json_path.clone(),
        },
    })
}

#[derive(Debug, Clone)]
pub struct ConfigModification {
    pub app_name: String,
    pub action: ConfigAction,
}

#[derive(Debug, Clone)]
pub enum ConfigAction {
    /// 自动修改（A 级）：备份原文件 → 修改 base_url → 不修改 api_key。
    AutoModify {
        config_path: String,
        base_url_json_path: String,
        new_base_url: String,
        api_key_json_path: Option<String>,
    },
    /// 显示 Diff 确认（B 级）。
    ShowDiff {
        config_path: String,
        original_base_url: String,
        new_base_url: String,
        diff_lines: Vec<String>,
    },
    /// 显示教程和复制按钮（C 级）。
    ShowTutorial {
        steps: Vec<String>,
        copy_text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_all_apps() {
        let apps = detect_installed_apps();
        assert_eq!(apps.len(), 4);
        assert!(apps.iter().any(|a| a.name == "ChatBox"));
        assert!(apps.iter().any(|a| a.name == "Cherry Studio"));
        assert!(apps.iter().any(|a| a.name == "Cursor"));
        assert!(apps.iter().any(|a| a.name == "Claude Code"));
    }

    #[test]
    fn claude_code_gets_tutorial() {
        let app = DetectedApp {
            name: "Claude Code".to_string(),
            config_path: "/tmp/.claude/settings.json".to_string(),
            level: "C".to_string(),
            configured: false,
            current_base_url: None,
            suggested_base_url: "http://127.0.0.1:10520".to_string(),
            api_key_present: false,
            original_base_url: None,
            base_url_json_path: None,
            api_key_json_path: None,
        };
        let modif = get_config_modification(&app).unwrap();
        match modif.action {
            ConfigAction::ShowTutorial { steps, copy_text } => {
                assert!(!steps.is_empty());
                assert_eq!(copy_text, "http://127.0.0.1:10520");
            }
            _ => panic!("expected ShowTutorial"),
        }
    }
}
