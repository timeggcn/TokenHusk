//! JSON 结构噪声压缩（PoC）。
//!
//! 设计意图（与 AGENTS.md Rule 3「零语义篡改」一致）：
//!   只清理「结构噪声」，不动语义内容；纯函数、确定性，绝不调用 LLM。
//!
//! 规则（与用户确认的 PoC 范围一致）：
//!   1. 去除值为 `null` / `{}` / `[]` 的字段（保留空字符串、`0`、`false` —— 语义风险）。
//!   2. 数组去重：按 `serde_json::Value` 全等判等，保序保留首次出现（确定性）。
//!   3. 数组截断：长度 > 10 时保留前 10 项 + 兄弟字段 `<key>_count = N`（N 为去重后长度）。
//!   4. 顺序：先去重，再截断（去重后可能已 ≤10，节省一次截断）。
//!   5. 仅对对象字段值是数组的情况插入 `_count`；根级数组仅去重，不补 count（无可挂载的兄弟键）。
//!   6. 递归处理所有嵌套结构。
//!
//! 确定性保证：
//!   - `serde_json::Map` 默认即有序（BTreeMap），序列化输出 key 顺序稳定；
//!   - 去重用 HashSet<Value> 保留首次出现 —— 同输入永远同输出。

use std::collections::HashSet;

use serde_json::{Map, Value};

/// 单字段数组截断阈值（与用户确认的「超过 10 项」一致）。
pub const MAX_ARRAY_ITEMS: usize = 10;

/// PoC 入口：压缩 JSON 结构噪声。
///
/// 复杂度：O(n) 节点访问 + O(k²) 数组去重（k = 单数组长度；Value 哈希常量级）。
/// Phase 0 够用；Phase 1 可改为流式 / 引用去重。
pub fn json_crusher_poc(input: &Value) -> Value {
    crush(input)
}

fn crush(value: &Value) -> Value {
    match value {
        Value::Object(map) => crush_object(map),
        // 数组统一去重（保序）；是否截断由父节点决定（仅对象字段数组会截断+补 count）。
        Value::Array(items) => Value::Array(crush_array(items)),
        leaf => leaf.clone(),
    }
}

fn crush_object(map: &Map<String, Value>) -> Value {
    let mut out: Map<String, Value> = Map::new();
    for (k, v) in map {
        let cleaned = crush(v);
        if is_structural_noise(&cleaned) {
            continue;
        }
        // 截断 + count 兄弟字段在此插入：只有对象字段才知道 key 名，且 cleaned 已去重。
        if let Value::Array(items) = cleaned {
            if items.len() > MAX_ARRAY_ITEMS {
                out.insert(format!("{k}_count"), Value::Number(items.len().into()));
                let truncated: Vec<Value> = items.into_iter().take(MAX_ARRAY_ITEMS).collect();
                out.insert(k.clone(), Value::Array(truncated));
            } else {
                out.insert(k.clone(), Value::Array(items));
            }
        } else {
            out.insert(k.clone(), cleaned);
        }
    }
    Value::Object(out)
}

fn crush_array(items: &[Value]) -> Vec<Value> {
    let cleaned: Vec<Value> = items.iter().map(crush).collect();
    dedup_preserving_first(cleaned)
}

fn dedup_preserving_first(items: Vec<Value>) -> Vec<Value> {
    let mut seen: HashSet<Value> = HashSet::with_capacity(items.len());
    let mut out: Vec<Value> = Vec::with_capacity(items.len());
    for item in items {
        if seen.contains(&item) {
            continue;
        }
        // 仅对保留的项克隆进 set，丢掉的不增加额外分配。
        seen.insert(item.clone());
        out.push(item);
    }
    out
}

/// 结构性噪声：null / 空对象 / 空数组。空字符串、`0`、`false` 不算。
fn is_structural_noise(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Object(m) => m.is_empty(),
        Value::Array(a) => a.is_empty(),
        _ => false,
    }
}

// ───────────────────── 测试 ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn removes_null_and_empty_containers() {
        let input = json!({
            "a": null,
            "b": {},
            "c": [],
            "keep_str": "",
            "keep_zero": 0,
            "keep_false": false,
            "nested": { "x": null, "y": "ok" }
        });
        let out = json_crusher_poc(&input);
        assert_eq!(out, json!({
            "keep_str": "",
            "keep_zero": 0,
            "keep_false": false,
            "nested": { "y": "ok" }
        }));
    }

    #[test]
    fn dedup_preserves_first_occurrence_order() {
        let input = json!({ "tags": ["a", "b", "a", "c", "b", "d"] });
        let out = json_crusher_poc(&input);
        assert_eq!(out, json!({ "tags": ["a", "b", "c", "d"] }));
    }

    #[test]
    fn truncates_over_max_with_count_sibling() {
        let items: Vec<Value> = (0..15).map(|i| json!({"id": i})).collect();
        let input = json!({ "users": items });
        let out = json_crusher_poc(&input);
        let users = out.get("users").and_then(|v| v.as_array()).expect("users");
        assert_eq!(users.len(), MAX_ARRAY_ITEMS);
        assert_eq!(
            out.get("users_count").and_then(|v| v.as_u64()),
            Some(items.len() as u64)
        );
    }

    #[test]
    fn dedup_before_truncate_may_skip_truncation() {
        // 12 项其中 3 对重复 → 去重 9 项 → 不触发截断，无 _count。
        let mut items: Vec<Value> = vec![];
        for i in 0..9 {
            items.push(json!({"id": i}));
        }
        for i in 0..3 {
            items.push(json!({"id": i}));
        }
        let input = json!({ "items": items });
        let out = json_crusher_poc(&input);
        assert_eq!(out.get("items_count"), None);
        assert_eq!(out.get("items").and_then(|v| v.as_array()).unwrap().len(), 9);
    }

    #[test]
    fn nested_arrays_and_objects_processed() {
        let input = json!({
            "level1": {
                "level2": {
                    "list": [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6],
                    "null_field": null,
                    "empty_obj": {}
                }
            }
        });
        let out = json_crusher_poc(&input);
        let list = out.pointer("/level1/level2/list").and_then(|v| v.as_array()).unwrap();
        assert_eq!(list, &vec![json!(1), json!(2), json!(3), json!(4), json!(5), json!(6)]);
        assert!(out.pointer("/level1/level2/null_field").is_none());
        assert!(out.pointer("/level1/level2/empty_obj").is_none());
    }

    #[test]
    fn root_array_dedups_without_count() {
        let input = json!(["x", "y", "x", "z"]);
        let out = json_crusher_poc(&input);
        assert_eq!(out, json!(["x", "y", "z"]));
    }

    #[test]
    fn is_deterministic() {
        let input = json!({ "a": [1, 1, null, {}, [], "k"], "b": { "c": [9, 9] } });
        let out1 = serde_json::to_string(&json_crusher_poc(&input)).unwrap();
        let out2 = serde_json::to_string(&json_crusher_poc(&input)).unwrap();
        assert_eq!(out1, out2);
    }

    #[test]
    fn fixture_compression_ratio_over_30_percent() {
        // 模拟「50 个用户、每项带 meta:null」风格的工具输出（plan.md §3.2 示例）。
        let mut users: Vec<Value> = Vec::with_capacity(50);
        for i in 0..50 {
            users.push(json!({
                "id": i,
                "name": format!("user_{i}"),
                "email": format!("user_{i}@example.com"),
                "meta": null,
            }));
        }
        let input = json!({
            "status": "success",
            "data": {
                "users": users,
                "pagination": { "page": 1, "total": 50, "per_page": 50 },
                "debug": {},
                "trace_id": "abc-123-def"
            }
        });
        let before = serde_json::to_string(&input).unwrap().len();
        let after = serde_json::to_string(&json_crusher_poc(&input)).unwrap().len();
        let ratio = 1.0 - (after as f64) / (before as f64);
        assert!(
            ratio > 0.30,
            "expected compression ratio > 30%, got {:.1}% (before={}, after={})",
            ratio * 100.0,
            before,
            after
        );
    }

    #[test]
    fn scalars_passthrough() {
        assert_eq!(json_crusher_poc(&json!(null)), Value::Null);
        assert_eq!(json_crusher_poc(&json!(true)), Value::Bool(true));
        assert_eq!(json_crusher_poc(&json!(42)), json!(42));
        assert_eq!(json_crusher_poc(&json!("s")), json!("s"));
    }
}
