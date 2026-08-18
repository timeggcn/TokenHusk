# TokenHusk v1.0 — 修订版完整开发方案

> **定位**：AI Agent 的成本控制层（不是 Token 压缩工具）
> **核心哲学**：帮你把 Prompt 里的垃圾扔掉，而不是帮你写更好的 Prompt
> **辩论结论**：聚焦结构噪声清理 + 费用观测 + 输出约束，与自然语言压缩和模型路由保持距离

---

## 一、产品重新定位

### 1.1 一句话定义

```
旧定位：Token 节省工具（压缩 Prompt 省钱）
新定位：AI Agent 成本控制层（观测 + 治理 + 清理）

类比：
  旧 = 一个"省水龙头"
  新 = 一个"家庭用水管理系统"（水表 + 漏水检测 + 阀门控制 + 用水报告）
```

### 1.2 核心价值主张

| 层级 | 价值 | 用户感知 |
|------|------|----------|
| **L1 观测** | 知道钱花在哪 | "原来 60% 的 Token 是工具输出" |
| **L2 治理** | 自动清理结构噪声 | "什么都不用做，自动省了 40%" |
| **L3 约束** | 控制输出和调用行为 | "回答更简洁了，不再废话连篇" |
| **L4 缓存** | 重复请求不再付费 | "同样的问题不用重复花钱" |

### 1.3 目标用户（辩论共识）

```
✅ 核心用户（月省 > $50，强需求）：
─────────────────────────────────────────
• Claude Code / Cursor Agent 重度用户（API 按量计费）
• 日调用 > 50 次，单次上下文 > 20K token
• 使用无 Prompt Caching 的 Provider（DeepSeek/通义/本地）
• 多工具并行使用（需要统一成本视图）

⚠️ 次要用户（月省 $10-50，弱需求）：
─────────────────────────────────────────
• 已有 Prompt Caching 但想进一步优化
• 团队/小工作室（需要配额管理）

❌ 非目标用户（不值得服务）：
─────────────────────────────────────────
• 订阅制用户（省 token ≠ 省钱）
• 纯聊天用户（上下文短，无结构噪声）
• 月消费 < $20 的轻度用户
```

### 1.4 与 Headroom 的关系（辩论共识）

```
Headroom = 压缩引擎（一把螺丝刀）
TokenHusk = 成本管理平台（工具箱 + 工作台 + 仪表盘）

技术关系：
• 借鉴 Headroom 的压缩算法思路（自行实现）
• 不复制 Headroom 的代码（Apache 2.0 合规）
• 增加 Headroom 没有的：观测层、缓存层、GUI、配置助手
• 长期：可作为 Headroom 的 GUI 前端（社区合作）
```

---

## 二、系统架构（修订版）

### 2.1 总体架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     TokenHusk Desktop (Tauri 2.0)                          │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Frontend (React + Tailwind)                     │  │
│  │                                                                       │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐   │  │
│  │  │实时流量  │ │成本仪表盘│ │策略面板  │ │请求日志  │ │ 配置向导    │   │  │
│  │  │监控面板  │ │& 报告   │ │& 预设   │ │& Diff   │ │ (Layer 0)  │   │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                              ↕ Tauri IPC                                     │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Rust Core                                       │  │
│  │                                                                       │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  ① Proxy Server (axum)                                          │  │  │
│  │  │  • HTTP 监听 localhost:10520                                    │  │  │
│  │  │  • 服务商路由 & Adapter                                         │  │  │
│  │  │  • SSE 流式透传                                                 │  │  │
│  │  │  • ⚡ 健康检查端点 /health                                      │  │  │
│  │  │  • ⚡ 超时降级（>50ms 跳过压缩直接转发）                         │  │  │
│  │  │  • ⚡ 进程守护 + 自动重启                                       │  │  │
│  │  └──────────────────────────────┬──────────────────────────────────┘  │  │
│  │                                 │                                     │  │
│  │  ┌──────────────────────────────▼──────────────────────────────────┐  │  │
│  │  │  ② Observation Layer（观测层）🆕                                 │  │  │
│  │  │  • 每次请求记录：原始 token / 压缩后 token / 耗时 / 服务商       │  │  │
│  │  │  • 按工具/模型/时段聚合统计                                      │  │  │
│  │  │  • 费用估算（各模型单价表）                                      │  │  │
│  │  │  • 异常检测（token 突增告警）                                    │  │  │
│  │  └──────────────────────────────┬──────────────────────────────────┘  │  │
│  │                                 │                                     │  │
│  │  ┌──────────────────────────────▼──────────────────────────────────┐  │  │
│  │  │  ③ Compression Pipeline（压缩管线）                              │  │  │
│  │  │                                                                 │  │  │
│  │  │  Stage A: CacheAligner（Provider 缓存对齐）                      │  │  │
│  │  │     → 稳定前缀 / 动态字段后置                                    │  │  │
│  │  │                                                                 │  │  │
│  │  │  Stage B: StructureCleaner（结构噪声清理）⭐ 核心                 │  │  │
│  │  │     ├─ JSONCrusher: null/空值/重复结构                          │  │  │
│  │  │     ├─ LogDeduplicator: 日志去重/模板化                         │  │  │
│  │  │     ├─ ToolOutputTrimmer: 工具输出截断/摘要                      │  │  │
│  │  │     └─ CodeContextReducer: 非目标代码精简                       │  │  │
│  │  │                                                                 │  │  │
│  │  │  Stage C: ContextManager（上下文管理）                           │  │  │
│  │  │     ├─ SlidingWindow: 对话历史裁剪                              │  │  │
│  │  │     ├─ ImportanceScorer: 关键轮次保留                           │  │  │
│  │  │     └─ SummaryReplacer: 旧轮次→摘要                            │  │  │
│  │  │                                                                 │  │  │
│  │  │  Stage D: OutputConstraint（输出约束）                           │  │  │
│  │  │     ├─ MaxTokensLimiter                                         │  │  │
│  │  │     ├─ BrevityInjector: "简洁回答"指令                          │  │  │
│  │  │     └─ FormatConstraint: 格式约束                               │  │  │
│  │  │                                                                 │  │  │
│  │  │  ⚠️ 安全阀:                                                     │  │  │
│  │  │     • 压缩后 token ≥ 原始 → 跳过压缩                            │  │  │
│  │  │     • 任何 Stage 异常 → fallback 到原始请求                      │  │  │
│  │  │     • 总处理时间 > 50ms → 跳过压缩直接转发                       │  │  │
│  │  └──────────────────────────────┬──────────────────────────────────┘  │  │
│  │                                 │                                     │  │
│  │  ┌──────────────────────────────▼──────────────────────────────────┐  │  │
│  │  │  ④ Quality Guard（质量守护）🆕                                   │  │  │
│  │  │  • 压缩前后 Diff 记录（可审查）                                  │  │  │
│  │  │  • 用户反馈按钮（"这次回答变差了"）                              │  │  │
│  │  │  • 自动降级：连续 3 次负反馈 → 降低压缩强度                      │  │  │
│  │  │  • Golden Test：内置测试集验证压缩质量                           │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                       │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  ⑤ Config Assistant（配置助手 - Layer 0）                        │  │  │
│  │  │  • 检测已安装 AI 应用                                           │  │  │
│  │  │  • 安全修改 API Base URL（仅支持验证过的应用）                    │  │  │
│  │  │  • 未验证应用 → 提供"复制地址 + 图文教程"                       │  │  │
│  │  │  • 配置备份 + 一键还原 + 紧急还原按钮                           │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                       │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  ⑥ Fallback & Safety（安全兜底）🆕                              │  │  │
│  │  │  • 代理不可用 → 自动切换直连（用户无感）                        │  │  │
│  │  │  • 一键全局关闭（系统托盘右键 → "暂停 TokenHusk"）            │  │  │
│  │  │  • 配置一键还原（恢复所有应用的原始 API 地址）                   │  │  │
│  │  │  • 压缩 Pipeline 任何异常 → 原样转发（绝不阻断请求）             │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 请求处理流程（含安全兜底）

```
用户应用发送请求
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ ① 接收请求                                              │
│    • 解析 JSON body                                     │
│    • 识别目标服务商（通过 URL path / 配置映射）           │
│    • 如果解析失败 → 原样转发，记录警告                    │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ ② 预检（<1ms）                                          │
│    • 请求体 > 1MB？→ 跳过压缩，直接转发                  │
│    • 用户已暂停？→ 直接转发                              │
│    • 该服务商已禁用压缩？→ 直接转发                       │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ ③ 压缩 Pipeline（带超时保护）                            │
│                                                         │
│    开始计时 ──────────────────────────────────┐         │
│    │                                         │         │
│    ▼                                         │         │
│    Stage A: CacheAligner (2-5ms)             │         │
│    │                                         │         │
│    ▼                                         │         │
│    Stage B: StructureCleaner (5-20ms)        │         │
│    │                                         │         │
│    ▼                                         │         │
│    Stage C: ContextManager (3-10ms)          │         │
│    │                                         │         │
│    ▼                                         │         │
│    Stage D: OutputConstraint (1-2ms)         │         │
│    │                                         │         │
│    ▼                                         │         │
│    验证：压缩后 token < 原始？                │         │
│    │  否 → 使用原始请求                       │         │
│    │  是 → 使用压缩后请求                     │         │
│    │                                         │         │
│    总耗时 > 50ms？                            │         │
│    │  是 → 中止压缩，使用原始请求              │         │
│    │  否 → 继续                              │         │
│    │                                         │         │
│    ─────────────────────────────────────────┘         │
│                                                         │
│    ⚠️ 任何 Stage 抛出异常 → catch → 使用原始请求         │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ ④ 转发到上游 API                                        │
│    • 流式透传 SSE 响应                                   │
│    • 记录响应中的 usage 字段（实际 token 消耗）           │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ ⑤ 记录 & 统计                                           │
│    • 原始 token / 压缩后 token / 节省比例                │
│    • 费用估算                                            │
│    • 耗时                                                │
│    • 异步写入 SQLite（不阻塞响应）                       │
└─────────────────────────────────────────────────────────┘
```

### 2.3 开源（修订版）

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  📗 开源 (Apache 2.0)                                         │
│  ─────────────                                         │
│                                                                         │
│  ✅ 完整代理服务器                                        │
│  ✅ 全部服务商 Adapter                              │
│  ✅ Stage A: CacheAligner                                               │
│  ✅ Stage B: StructureCleaner 全部                       │
│     • JSONCrusher                                   │
│     • LogDeduplicator                                                  │
│     • ToolOutputTrimmer                          │
│     • CodeContextReducer                                 │
│  ✅ Stage C: ContextManager                                              │
│  ✅ Stage D: OutputConstraint                          │
│  ✅ Token 计数器                                    │
│  ✅ 观测层（基础统计）                                                   │
│  ✅ 请求日志 & Diff 查看                                      │
│  ✅ 配置助手 (Layer 0)                        │
│  ✅ 安全兜底 & Fallback                                                  │
│  ✅ 桌面 UI                                               │
│  ✅ 质量守护（基础版）                                                   │
│                                                            │
│                                                                         │
│  开源版独立价值：                                                        │
│  "装完即用，自动省 30-50% Token，                                        │
│   有实时仪表盘，有安全兜底"                                              │
│                                                                         │
│                                                          │
│                                          │
│                                                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**修订要点**：开源版包含**完整的压缩 Pipeline**，不再是"几行代码就能实现的事"。
---

## 三、压缩 Pipeline 详细设计（修订版）

### 3.1 Stage A: CacheAligner

```rust
/// 目标：最大化 Provider 侧 KV Cache 命中率
/// 辩论结论：与 Provider Caching 互补，不竞争

pub struct CacheAligner {
    provider: ProviderType,
}

impl CacheAligner {
    pub fn align(&self, request: &mut UnifiedRequest) -> AlignResult {
        match self.provider {
            // Anthropic: 需要显式 cache_control 标记
            ProviderType::Anthropic => {
                // 1. 识别 system prompt 中的稳定前缀
                // 2. 在稳定前缀末尾添加 cache_control: {"type": "ephemeral"}
                // 3. 动态字段（日期、session_id）移至最后一条 user message
                self.align_anthropic(request)
            }
            
            // OpenAI: 自动前缀匹配（>1024 token），无需显式标记
            // 但仍需确保前缀稳定
            ProviderType::OpenAI => {
                // 1. 检测 system prompt 中的动态内容
                // 2. 将动态内容移至消息末尾
                // 3. 确保前 1024 token 在多次请求间一致
                self.align_openai(request)
            }
            
            // DeepSeek / 通义 / 本地: 无 Prompt Caching
            // CacheAligner 无操作，跳过
            _ => AlignResult::Skip,
        }
    }
    
    fn align_anthropic(&self, request: &mut UnifiedRequest) -> AlignResult {
        // 关键：不改变语义，只调整结构
        // 动态字段不能删除，只能移位
        
        // 示例：
        // Before: "Today is 2026-08-17. You are a helpful assistant..."
        // After:  "You are a helpful assistant... [Date: 2026-08-17]"
        // + cache_control 标记在 "You are a helpful assistant..." 之后
        
        // ⚠️ 安全规则：如果动态字段是语义关键（如"根据今天的日期计算"），
        //    则不移位，保持原样
        todo!()
    }
}
```

### 3.2 Stage B: StructureCleaner（核心）

```rust
/// 辩论共识：这是工具的"基本盘"，确定性最高、质量风险最低
/// 
/// 设计原则：
/// 1. 只清理"结构噪声"，不动"语义内容"
/// 2. 每种清理策略独立可开关
/// 3. 每种策略有"保守/标准/激进"三档
/// 4. 清理后必须可逆（记录被清理的内容摘要）

pub struct StructureCleaner {
    config: StructureCleanConfig,
}

pub struct StructureCleanConfig {
    pub json_crusher: JsonCrushConfig,
    pub log_dedup: LogDedupConfig,
    pub tool_trimmer: ToolTrimmerConfig,
    pub code_reducer: CodeReduceConfig,
}

// ─── JSON Crusher ───────────────────────────────────────────────────────

pub struct JsonCrushConfig {
    pub remove_null_fields: bool,        // 默认 true
    pub remove_empty_arrays: bool,       // 默认 true
    pub deduplicate_arrays: bool,        // 默认 true（保守模式关闭）
    pub flatten_depth: u32,              // 默认 3（超过此深度扁平化）
    pub truncate_strings_at: usize,      // 默认 500 字符
    pub max_array_items: usize,          // 默认 10（超过截断 + count）
    pub level: AggressivenessLevel,      // Conservative / Standard / Aggressive
}

impl JsonCrusher {
    /// 压缩 JSON 结构噪声
    /// 
    /// 示例（Standard 模式）：
    /// 
    /// Before (1,247 tokens):
    /// {
    ///   "status": "success",
    ///   "data": {
    ///     "users": [
    ///       {"id": 1, "name": "Alice", "email": "alice@...", "meta": null},
    ///       {"id": 2, "name": "Bob", "email": "bob@...", "meta": null},
    ///       ... (50 items, all with meta: null)
    ///     ],
    ///     "pagination": {"page": 1, "total": 50, "per_page": 50},
    ///     "debug": {},
    ///     "trace_id": "abc-123-def"
    ///   }
    /// }
    /// 
    /// After (312 tokens):
    /// {
    ///   "status": "success",
    ///   "data": {
    ///     "users": [{"id": 1, "name": "Alice", "email": "alice@..."}],
    ///     "users_count": 50,
    ///     "users_note": "49 more items with same schema",
    ///     "pagination": {"page": 1, "total": 50}
    ///   }
    /// }
    pub fn crush(&self, value: &mut serde_json::Value) -> CrushResult {
        let mut stats = CrushStats::new();
        self.crush_recursive(value, &mut stats, 0);
        CrushResult { stats }
    }
}

// ─── Log Deduplicator ───────────────────────────────────────────────────

pub struct LogDedupConfig {
    pub dedup_threshold: f32,       // 行相似度阈值（默认 0.8）
    pub keep_first_n: usize,        // 保留前 N 条（默认 3）
    pub keep_last_n: usize,         // 保留后 N 条（默认 2）
    pub fold_repetitions: bool,     // 折叠重复行（默认 true）
    pub strip_timestamps: bool,     // 统一时间戳（默认 true）
    pub strip_pids: bool,           // 统一进程/线程 ID（默认 true）
}

impl LogDeduplicator {
    /// 示例：
    /// 
    /// Before (890 tokens):
    /// [2026-08-17 10:23:45.123] INFO  [thread-42] UserService: login SUCCESS userId=12345
    /// [2026-08-17 10:23:45.124] INFO  [thread-42] UserService: login SUCCESS userId=12346
    /// [2026-08-17 10:23:45.125] INFO  [thread-42] UserService: login SUCCESS userId=12347
    /// ... (200 identical lines)
    /// [2026-08-17 10:23:46.001] ERROR [thread-42] UserService: login FAILED userId=99999
    /// 
    /// After (95 tokens):
    /// [INFO] UserService: login SUCCESS ×200 (userId=12345..12544)
    /// [ERROR] UserService: login FAILED userId=99999
    pub fn dedup(&self, content: &str) -> DedupResult {
        // 1. 按行分割
        // 2. 提取模板（去除变量部分：时间戳、ID、数字）
        // 3. 相同模板的行 → 保留首尾 + count
        // 4. 重新组装
        todo!()
    }
}

// ─── Tool Output Trimmer ────────────────────────────────────────────────

pub struct ToolTrimmerConfig {
    pub max_tool_output_tokens: usize,  // 默认 2000
    pub preserve_structure: bool,       // 保留结构（JSON keys / 代码签名）
    pub truncation_marker: String,      // "[... truncated, N more items ...]"
}

impl ToolOutputTrimmer {
    /// Agent 工具调用返回的结果往往巨大（文件内容、搜索结果、命令输出）
    /// 模型通常只需要前几百个 token 就能理解
    /// 
    /// 策略：
    /// • JSON 工具输出 → 走 JsonCrusher
    /// • 文件内容 → 保留前 N 行 + 后 M 行 + 中间省略
    /// • 搜索结果 → 保留 top-3 + count
    /// • 命令输出 → 保留首行 + 尾行 + 错误行
    pub fn trim(&self, tool_output: &str, tool_name: &str) -> TrimResult {
        todo!()
    }
}

// ─── Code Context Reducer ───────────────────────────────────────────────

pub struct CodeReduceConfig {
    pub remove_comments: bool,          // 默认 true
    pub remove_empty_lines: bool,       // 默认 true
    pub collapse_imports: bool,         // 折叠 import 块（默认 true）
    pub keep_function_bodies: bool,     // ⚠️ 默认 TRUE（辩论共识：不删函数体）
    pub max_file_tokens: usize,         // 单文件最大 token（默认 3000）
    pub target_file_full: bool,         // 目标文件保持完整
}

impl CodeContextReducer {
    /// 辩论共识：不激进删除函数体（模型需要实现细节）
    /// 
    /// 安全策略：
    /// • 只移除注释和空行（零风险）
    /// • 折叠 import 块为摘要（低风险）
    /// • 非目标文件：保留签名 + 首行实现（中风险，默认关闭）
    /// • 目标文件：保持完整（零风险）
    /// 
    /// Before:
    /// ```python
    /// # This function handles user authentication
    /// # It checks credentials against the database
    /// # Returns True if valid, False otherwise
    /// import os
    /// import sys
    /// import json
    /// import logging
    /// from typing import Optional, Dict, List
    /// from datetime import datetime
    /// 
    /// def authenticate(username: str, password: str) -> bool:
    ///     ...actual implementation...
    /// ```
    /// 
    /// After:
    /// ```python
    /// import os, sys, json, logging
    /// from typing import Optional, Dict, List
    /// from datetime import datetime
    /// 
    /// def authenticate(username: str, password: str) -> bool:
    ///     ...actual implementation...
    /// ```
    pub fn reduce(&self, code: &str, lang: Language, is_target: bool) -> ReduceResult {
        todo!()
    }
}
```

### 3.3 Stage C: ContextManager

```rust
/// 对话历史管理
/// 辩论共识：滑动窗口 + 重要性评分是安全的
/// 摘要替代有轻微风险，默认关闭

pub struct ContextManager {
    config: ContextConfig,
}

pub struct ContextConfig {
    pub strategy: ContextStrategy,
    pub max_history_tokens: usize,      // 总历史 token 上限（默认 20000）
    pub max_turns: usize,               // 最大保留轮数（默认 20）
    pub protect_recent: usize,          // 保护最近 N 轮不裁剪（默认 3）
    pub use_summary: bool,              // 是否用摘要替代（默认 false）
}

pub enum ContextStrategy {
    /// 简单滑动窗口：只保留最近 N 轮
    SlidingWindow,
    
    /// 重要性评分：保留包含关键信息的轮次
    /// 评分因素：
    /// • 包含代码块 → +3
    /// • 包含"决定"/"结论"/"最终" → +2
    /// • 包含错误信息/解决方案 → +2
    /// • 包含用户明确指令 → +2
    /// • 纯寒暄/确认 → -1
    ImportanceScored,
    
    /// Token 预算：按 token 预算从后往前填充
    TokenBudget,
}

impl ContextManager {
    pub fn manage(&self, messages: &mut Vec<Message>) -> ManageResult {
        let system_msg = messages.first().cloned(); // system prompt 永远保留
        
        // 1. 分离 system / user / assistant 消息
        // 2. 从最后一条往前计算 token
        // 3. 超过预算 → 裁剪最旧的轮次
        // 4. 如果开启摘要 → 被裁掉的轮次生成 1 句摘要放在最前
        // 5. 重新组装
        
        todo!()
    }
}
```

### 3.4 Stage D: OutputConstraint

```rust
/// 输出约束
/// 辩论共识：实现最简单，立即生效，零质量风险

pub struct OutputConstraint {
    config: OutputConfig,
}

pub struct OutputConfig {
    pub max_tokens: Option<u32>,         // 覆盖/限制 max_tokens
    pub inject_brevity: bool,            // 追加简洁指令
    pub brevity_text: String,            // 自定义简洁指令文本
    pub format_hint: Option<String>,     // 输出格式提示
}

impl OutputConstraint {
    pub fn apply(&self, request: &mut UnifiedRequest) -> ConstraintResult {
        // 1. max_tokens 覆盖
        if let Some(max) = self.config.max_tokens {
            request.max_tokens = Some(max.min(request.max_tokens.unwrap_or(u32::MAX)));
        }
        
        // 2. 简洁指令注入（追加到 system prompt 末尾）
        // 默认文本："请简洁回答，避免不必要的解释和重复。"
        // 用户可自定义
        if self.config.inject_brevity {
            if let Some(system) = request.messages.first_mut() {
                system.content.push_str(&format!("\n\n{}", self.config.brevity_text));
            }
        }
        
        ConstraintResult::Applied
    }
}
```

### 3.5 压缩策略预设（修订版）

| 预设 | 启用的 Stage | 适用场景 | 预期节省 | 质量风险 |
|------|-------------|----------|----------|----------|
| **保守模式** | A + D | 生产环境 / 质量敏感 | 10-20% | 极低 |
| **标准模式** ⭐默认 | A + B(保守) + C + D | 日常开发 | 30-50% | 低 |
| **激进模式** | A + B(激进) + C + D | 调试 / 成本敏感 | 50-70% | 中 |
| **自定义** | 用户自选 | 高级用户 | 可变 | 可变 |

---

## 四、观测层设计（新增核心模块）

### 4.1 数据模型

```rust
/// 每次 API 请求的完整记录
struct RequestRecord {
    id: u64,
    timestamp: DateTime<Utc>,
    
    // 来源
    source_app: String,          // "Claude Code" / "Cursor" / "ChatBox"
    provider: String,            // "anthropic" / "openai" / "deepseek"
    model: String,               // "claude-sonnet-4-20250514"
    
    // Token 统计
    original_input_tokens: u32,
    compressed_input_tokens: u32,
    output_tokens: u32,
    saved_tokens: u32,           // original - compressed
    saved_ratio: f32,            // 0.0 - 1.0
    
    // 费用估算
    estimated_cost_usd: f64,     // 按模型单价计算
    saved_cost_usd: f64,
    
    // 压缩详情
    stages_applied: Vec<String>, // ["CacheAligner", "JsonCrusher", ...]
    compression_time_ms: u32,
    skipped: bool,               // 是否跳过压缩（超时/异常）
    skip_reason: Option<String>,
    
    // 元数据
    message_count: u32,          // 消息条数
    has_code: bool,
    has_json: bool,
    has_log: bool,
}

/// 聚合统计
struct DailyStats {
    date: Date,
    total_requests: u32,
    total_original_tokens: u64,
    total_compressed_tokens: u64,
    total_saved_tokens: u64,
    total_estimated_cost: f64,
    total_saved_cost: f64,
    by_app: HashMap<String, AppStats>,
    by_provider: HashMap<String, ProviderStats>,
    by_stage: HashMap<String, StageStats>,  // 各压缩策略的贡献
}
```

### 4.2 仪表盘视图

```
┌─────────────────────────────────────────────────────────────────┐
│  TokenHusk Dashboard                                          │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  今日概览                                                │   │
│  │                                                         │   │
│  │  请求次数: 147     节省 Token: 892,340 (42%)           │   │
│  │                       │   │
│  │                                                         │   │
│  │  [🟢 代理运行中]  [⏸ 暂停]  [⚙ 设置]                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌──────────────────────────┐  ┌────────────────────────────┐  │
│  │  Token 消耗分布          │  │  节省来源分析               │  │
│  │                          │  │                            │  │
│  │  ████ 工具输出 58%       │  │  JSON 压缩    45%         │  │
│  │  ███  代码上下文 22%     │  │  日志去重     25%         │  │
│  │  ██   对话历史 12%       │  │  上下文裁剪   18%         │  │
│  │  █    用户指令 8%        │  │  输出约束     12%         │  │
│  │                          │  │                            │  │
│  │  💡 60% 的 Token 是      │  │  本月累计节省: $187.50     │  │
│  │     工具输出结构噪声      │  │  年化节省: $2,250          │  │
│  └──────────────────────────┘  └────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  最近请求                                                │   │
│  │                                                         │   │
│  │  10:23:45  Claude Code → anthropic  │ 42K→18K │ -57%  │   │
│  │  10:23:12  Claude Code → anthropic  │ 38K→15K │ -61%  │   │
│  │  10:22:58  Cursor → openai          │ 12K→8K  │ -33%  │   │
│  │  10:22:31  Claude Code → anthropic  │ 55K→55K │ 跳过  │   │
│  │                                                         │   │
│  │  [查看详情] [查看 Diff]                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  ⚠️ 质量反馈                                            │   │
│  │  最近 7 天: 👍 94% 正常  │  👎 6% 反馈质量下降          │   │
│  │  自动调整: JsonCrusher 已从"激进"降为"标准"             │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 五、质量守护系统（新增）

### 5.1 设计原则

```
辩论共识：
• 压缩质量退化是"静默的、渐进的"
• 没有质量评估机制，用户不敢用
• 必须让用户能"看到"压缩了什么
• 必须让用户能"反馈"质量变差了
• 系统必须能"自适应"调整压缩强度
```

### 5.2 三层质量守护

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: 压缩前验证（自动）                                      │
│ ─────────────────────────────────────                            │
│ • 压缩后 token ≥ 原始 → 跳过压缩                                │
│ • 压缩后 JSON 格式不合法 → 回滚该条消息                          │
│ • 压缩后消息结构不完整（缺少 role/content）→ 回滚                │
│ • System Prompt 被修改 → 警告（默认不修改 system prompt 内容）   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: 压缩后可审查（用户）                                    │
│ ─────────────────────────────────────                            │
│ • 每次请求的 Diff 可查看（原始 vs 压缩后）                       │
│ • 高亮显示被移除的内容                                           │
│ • 支持"这次不要压缩"（单次跳过）                                 │
│ • 支持"这类内容不要压缩"（按类型禁用）                           │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: 自适应调整（闭环）                                      │
│ ─────────────────────────────────────                            │
│ • 用户点击 👎 → 记录负反馈                                      │
│ • 连续 3 次负反馈 → 自动降低压缩强度一级                         │
│ • 连续 5 次负反馈 → 暂停该 Stage，通知用户                       │
│ • 用户可手动恢复                                                │
│                                                                 │
│ • 可选：内置 Golden Test                                        │
│   - 10 个标准 Prompt（代码生成/问答/分析）                       │
│   - 压缩前后分别调用模型                                         │
│   - 对比回答质量（简单字符串相似度 + 长度比）                     │
│   - 质量下降 > 20% → 告警                                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 六、配置助手（Layer 0）修订版

### 6.1 安全分级（辩论共识）

```
┌─────────────────────────────────────────────────────────────────┐
│ 配置助手安全分级                                                 │
│                                                                 │
│ 🟢 A级（自动修改，已充分验证）：                                  │
│    • ChatBox: ~/.config/chatbox/config.json                      │
│    • Cherry Studio: ~/.config/cherry-studio/settings.json        │
│    验证方式：开源项目，配置文件格式稳定，社区确认                  │
│                                                                 │
│ 🟡 B级（半自动，修改前确认）：                                    │
│    • Cursor: ~/.cursor/settings.json                             │
│    • VS Code (Continue): ~/.continue/config.json                 │
│    验证方式：需要用户确认，修改前显示 Diff                        │
│                                                                 │
│ 🔴 C级（仅提供教程，不自动修改）：                                │
│    • Claude Code: 需要修改环境变量或 MCP 配置                    │
│    • Copilot: 不支持自定义 API 地址                              │
│    • LobeChat / NextChat: Web 应用，需在设置中手动改             │
│    处理方式：提供一键复制地址 + 图文/视频教程                     │
│                                                                 │
│ ⚠️ 所有级别通用规则：                                            │
│    • 修改前必须备份原文件（.bak + 时间戳）                       │
│    • 提供"一键还原"按钮（系统托盘 + 主界面）                     │
│    • 修改后自动验证（发一个测试请求确认连通）                     │
│    • 验证失败 → 自动回滚                                        │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 配置流程

```
首次启动
    │
    ▼
┌─────────────────────────────┐
│ Step 1: 检测已安装的 AI 应用  │
│ • 扫描常见路径               │
│ • 检测环境变量               │
│ • 检测进程                   │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Step 2: 显示检测结果          │
│ "检测到以下应用："           │
│ ✅ ChatBox (已安装)          │
│ ✅ Cherry Studio (已安装)    │
│ ⚠️ Cursor (已安装，需确认)   │
│ 其他无法通用的，手动配置  │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Step 3: 配置服务商            │
│ "你要连接哪个 AI 服务？"     │
│ • Anthropic (Claude)         │
│ • OpenAI                     │
│ • DeepSeek                   │
│ • 通义千问                   │
│ • 自定义 (OpenAI 兼容)       │
│                              │
│ [输入 API Key]               │
│ ⚠️ Key 存储在系统 Keychain   │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Step 4: 选择压缩预设          │
│ "推荐：标准模式"             │
│ ○ 观测模式（只记录不压缩）    │
│ ○ 保守模式                   │
│ ● 标准模式 ⭐推荐            │
│ ○ 激进模式                   │
│ ○ 自定义                     │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Step 5: 执行配置              │
│ • 备份原配置 ✅              │
│ • 修改 API 地址 ✅           │
│ • 发送测试请求 ✅            │
│ • 配置完成！                 │
│                              │
│ "现在可以正常使用你的 AI 应用 │
│  TokenHusk 会在后台自动    │
│  帮你节省 Token。"           │
│                              │
│ [打开仪表盘] [完成]          │
└─────────────────────────────┘
```

---

## 七、安全兜底设计（辩论共识 P0）

### 7.1 Fallback 机制

```rust
/// 核心原则：TokenHusk 永远不应该阻断用户的 AI 请求

enum FallbackTrigger {
    ProxyCrashed,           // 代理进程崩溃
    PipelineTimeout,        // 压缩超时 (>50ms)
    PipelineError,          // 压缩异常
    CompressedLargerThanOriginal, // 压缩后反而更大
    UserPaused,             // 用户手动暂停
}

/// 实现方式：
/// 1. 应用配置中同时存储两个地址：
///    primary: http://localhost:10520/v1  (TokenHusk)
///    fallback: https://api.anthropic.com/v1  (直连)
///
/// 2. TokenHusk 代理提供 /health 端点
///    应用发请求前先 GET /health（或首次失败后切换）
///
/// 3. 如果 /health 失败 → 应用自动切换到 fallback 地址
///    （需要应用支持，或提供配置脚本）
///
/// 4. 对于不支持 fallback 的应用：
///    TokenHusk 内部保证"任何异常都原样转发"
///    绝不让 Pipeline 错误导致请求失败

/// 进程守护：
/// macOS: launchd plist (KeepAlive = true)
/// Windows: Windows Service / Task Scheduler
/// Linux: systemd unit (Restart = always)
```

### 7.2 一键操作

```
系统托盘菜单：
┌─────────────────────────────────┐
│ 🟢 TokenHusk 运行中           │
│                                 │
│ 今日节省: 892K tokens ($8.94)   │
│                                 │
│ ─────────────────────────       │
│ ⏸  暂停（所有请求直连）         │
│ 🔄 重启代理                     │
│ 📊 打开仪表盘                   │
│ ─────────────────────────       │
│ 🔙 紧急还原所有配置              │
│ ❌ 退出                         │
└─────────────────────────────────┘
```

---

## 八、技术选型（修订版）

| 层面 | 选择 | 理由 |
|------|------|------|
| 桌面框架 | Tauri 2.0 | 体积小、Rust 后端高性能 |
| 前端 | React 18 + Tailwind + Recharts | 快速开发仪表盘 |
| HTTP 代理 | Rust `axum` + `hyper` | 异步、SSE 原生支持 |
| Token 计数 | `tiktoken-rs` + 各模型 tokenizer | 精确计数 |
| JSON 处理 | `serde_json` | 高性能 |
| 代码解析 | `tree-sitter` | 多语言（仅用于注释/import 移除） |
| 本地存储 | SQLite (`rusqlite`) | 日志、统计 |
| 系统托盘 | `tray-icon` crate | 跨平台托盘 |
| 进程守护 | `launchd` / `systemd` / `WinService` | 自动重启 |
| API Key 存储 | 系统 Keychain (`keyring` crate) | 安全存储 |
| 打包 | Tauri bundler | .dmg / .msi / .AppImage |
| 分发 | **仅官网直接下载**（不走 App Store） | 沙箱限制 |

---

## 九、项目目录结构（修订版）

```
token-saver/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   │
│   │   ├── proxy/                      # 代理服务
│   │   │   ├── server.rs              # axum HTTP server
│   │   │   ├── router.rs             # 路由分发（tokenhusk.toml target 映射）
│   │   │   ├── stream.rs             # SSE 流式透传
│   │   │   ├── headers.rs            # Header 过滤 + sanitize_headers() 脱敏
│   │   │   ├── health.rs             # /health 端点
│   │   │   └── fallback.rs           # 降级 & 兜底
│   │   │
│   │   ├── adapter/                    # 服务商适配
│   │   │   ├── mod.rs
│   │   │   ├── openai.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── deepseek.rs
│   │   │   ├── tongyi.rs
│   │   │   ├── ollama.rs
│   │   │   └── custom.rs
│   │   │
│   │   ├── pipeline/                   # 压缩管线
│   │   │   ├── mod.rs                 # Pipeline 编排 + 超时 + fallback
│   │   │   ├── cache_aligner.rs       # Stage A
│   │   │   ├── structure/             # Stage B
│   │   │   │   ├── json_crusher.rs
│   │   │   │   ├── log_dedup.rs
│   │   │   │   ├── tool_trimmer.rs
│   │   │   │   └── code_reducer.rs
│   │   │   ├── context_manager.rs     # Stage C
│   │   │   └── output_constraint.rs   # Stage D
│   │   │
│   │   ├── observation/                # 观测层
│   │   │   ├── recorder.rs           # 请求记录
│   │   │   ├── aggregator.rs         # 聚合统计
│   │   │   ├── cost_calculator.rs    # 费用计算
│   │   │   └── anomaly.rs            # 异常检测
│   │   │
│   │   ├── quality/                    # 质量守护
│   │   │   ├── validator.rs          # 压缩前验证
│   │   │   ├── feedback.rs           # 用户反馈
│   │   │   └── auto_adjust.rs        # 自适应调整
│   │   │
│   │   ├── counter/                    # Token 计数
│   │   │   ├── tiktoken.rs
│   │   │   ├── anthropic.rs
│   │   │   └── estimator.rs
│   │   │
│   │   ├── storage/                    # 数据存储
│   │   │   ├── database.rs
│   │   │   ├── models.rs
│   │   │   └── migrations.rs
│   │   │
│   │   ├── config/                     # 配置管理
│   │   │   ├── app_config.rs
│   │   │   ├── providers.rs
│   │   │   └── strategies.rs
│   │   │
│   │   ├── assistant/                  # 配置助手
│   │   │   ├── detector.rs
│   │   │   ├── configurator.rs
│   │   │   ├── backup.rs
│   │   │   └── apps/
│   │   │       ├── chatbox.rs        # A级
│   │   │       ├── cherry_studio.rs  # A级
│   │   │       ├── cursor.rs         # B级
│   │   │       └── manual_guides.rs  # C级（教程）
│   │   │
│   │   ├── safety/                     # 安全兜底
│   │   │   ├── watchdog.rs           # 进程守护
│   │   │   ├── fallback.rs           # 降级策略
│   │   │   └── emergency_restore.rs  # 紧急还原
│   │   │
│   │   └── ipc/                        # Tauri IPC
│   │       ├── proxy_commands.rs
│   │       ├── stats_commands.rs
│   │       ├── config_commands.rs
│   │       └── quality_commands.rs
│   │
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                                # React 前端
│   ├── components/
│   │   ├── Dashboard/
│   │   │   ├── Overview.tsx           # 今日概览
│   │   │   ├── TokenDistribution.tsx  # Token 消耗分布
│   │   │   ├── SavingsChart.tsx       # 节省趋势图
│   │   │   └── RecentRequests.tsx     # 最近请求列表
│   │   ├── Strategy/
│   │   │   ├── PresetSelector.tsx     # 预设选择
│   │   │   ├── StageToggle.tsx        # 各 Stage 开关
│   │   │   └── AdvancedConfig.tsx     # 高级配置
│   │   ├── Logs/
│   │   │   ├── RequestDetail.tsx      # 请求详情
│   │   │   ├── DiffViewer.tsx         # 压缩前后 Diff
│   │   │   └── FeedbackButton.tsx     # 质量反馈
│   │   ├── Setup/
│   │   │   ├── SetupWizard.tsx        # 配置向导
│   │   │   ├── AppDetector.tsx        # 应用检测
│   │   │   └── ProviderForm.tsx       # 服务商配置
│   │   └── Safety/
│   │       ├── EmergencyRestore.tsx   # 紧急还原
│   │       └── PauseToggle.tsx        # 暂停开关
│   ├── App.tsx
│   └── main.tsx
│
├── tests/
│   ├── golden/                         # 质量评估 Golden Test
│   │   ├── code_generation/
│   │   ├── qa/
│   │   └── analysis/
│   ├── pipeline/                       # 压缩算法单元测试
│   ├── proxy/                          # 代理集成测试
│   └── fixtures/                       # 测试数据
│       ├── sample_requests/
│       └── app_configs/
│
├── docs/
├── scripts/
│   ├── install-daemon.sh              # 安装进程守护
│   └── uninstall.sh                   # 完整卸载 + 配置还原
│
├── LICENSE                             # Apache 2.0
└── README.md
```

---

## 十、开发里程碑（修订版：22-26 周）

### Phase 0：技术验证（2 周）

| 任务 | 产出 | 验证目标 |
|------|------|----------|
| Tauri 2.0 + React 骨架 | 可运行空壳 | 框架跑通 |
| axum 透明代理 → OpenAI API | /health + SSE 流式透传 | 代理可行性 |
| 路由配置模型 | tokenhusk.toml 读取 target 地址（§4.4） | 配置驱动转发 |
| Header 透传验证 + sanitize_headers() | byte-level 集成测试 + `#[must_use]` 脱敏函数 | 透传完整性 / 零泄露 |
| JSON 压缩 PoC | 去 null + 去重 + 截断 | 压缩率 > 30% |
| tiktoken-rs 计数 | cl100k vs o200k 对比 | 多编码计数验证 |
| Key Leak 检查 | 日志/DB 扫描 `sk-*` 模式 | 日志零泄露 |
| **关键验证：压缩后请求模型是否正常回答** | 10 个测试用例 | 质量可行性 |

> 备注（对齐 docs/prompt.md / README.md 更新）：
> - **Token 计数**：tiktoken-rs 无 Claude tokenizer（Anthropic 未开源），Phase 0 用
>   cl100k（GPT-4 系）vs o200k（GPT-4o 系）验证集成与多编码切换；Claude 近似计数归 Phase 3。
> - **Header 透传**：Authorization / X-Request-ID / 自定义头压缩前后 byte-level 对比
>   （AGENTS.md §7）；`sanitize_headers()` 为 `#[must_use]`，任何日志/DB 路径必须调用（AGENTS.md §5）。
> - **环境**：rust ≥ 1.95.0 / node ≥ 18.20.8（README.md 要求）。

### Phase 1：核心代理 + 基础压缩（3 周）

| 任务 | 产出 |
|------|------|
| 完整 HTTP 代理（axum） | localhost:18080 |
| OpenAI + Anthropic Adapter | 两大服务商 |
| SSE 流式透传 | 无感体验 |
| Stage B: JsonCrusher + LogDedup | 核心压缩 |
| Stage D: OutputConstraint | 输出限制 |
| Token 计数 + 基础统计 | 压缩前后对比 |
| **安全兜底：超时降级 + 异常 fallback** | 绝不阻断请求 |
| CLI 验证（curl 测试） | 端到端跑通 |

### Phase 2：桌面 UI + 配置助手（4 周）

| 任务 | 产出 |
|------|------|
| 仪表盘 UI（今日概览 + 趋势图） | 用户感知价值 |
| 请求日志 + Diff 查看 | 可审查性 |
| 配置向导（检测 → 配置 → 验证） | Layer 0 体验 |
| ChatBox + Cherry Studio 自动配置 | A 级应用 |
| Cursor 半自动配置 | B 级应用 |
| Claude Code 教程 | C 级引导 |
| 系统托盘 + 暂停/还原 | 安全兜底 |
| 进程守护（launchd / systemd） | 自动重启 |

### Phase 3：完整压缩引擎（4 周）

| 任务 | 产出 |
|------|------|
| Stage A: CacheAligner | Provider 缓存对齐 |
| Stage B: ToolOutputTrimmer | Agent 工具输出裁剪 |
| Stage B: CodeContextReducer | 代码注释/import 清理 |
| Stage C: ContextManager | 对话历史裁剪 |
| DeepSeek + 通义 + Ollama Adapter | 更多服务商 |
| 策略预设系统 | 观测/保守/标准/激进 |
| 各 Stage 独立开关 | 细粒度控制 |

### Phase 4：质量守护 + 打磨（3 周）

| 任务 | 产出 |
|------|------|
| 质量验证层（压缩后检查） | 自动回滚 |
| 用户反馈机制（👍👎） | 闭环 |
| 自适应调整（连续负反馈→降级） | 智能调整 |
| Golden Test 套件 | 回归测试 |
| 性能优化（大 Prompt 场景） | <50ms 保证 |
| 跨平台打包测试 | macOS/Win/Linux |
| 自动更新机制 | Tauri updater |

### Phase 5：开源发布（2 周）

| 任务 | 产出 |
|------|------|
| 文档 + README + 教程视频 | 用户引导 |
| GitHub 开源发布 | v1.0.0 |
| 官网 + 下载页 | 品牌 |
| Beta 测试（20-50 用户） | 反馈收集 |
| Bug 修复 + 优化 | 稳定性 |


### Phase 7：团队版 + 运营（持续）

| 任务 | 产出 |
|------|------|
| 团队管理功能 | Team 版 |
| 云端策略同步 | 增值服务 |
| 社区运营 | 用户增长 |
| 策略市场 | 生态 |

> **总计：22-26 周到 v1.0 发布**（辩论修正后的合理估算）

---

---
