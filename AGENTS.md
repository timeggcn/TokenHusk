# TokenHusk v1.0 - Project Constraints & Conventions

## 1. Project Overview

TokenHusk 是一个基于 Tauri 2.0 的 AI Agent 成本控制层桌面应用。
它通过本地 HTTP 代理拦截 LLM API 请求，使用确定性的规则引擎压缩 Prompt 中的冗余信息
（如巨大的 JSON 工具输出、重复日志、无关代码上下文），从而降低 Token 消耗和响应延迟。

**一句话定位**：帮你把 Prompt 里的垃圾扔掉，而不是帮你写更好的 Prompt。
**代理模式**：透明代理（Passthrough Proxy）。TokenHusk 不持有、不存储、不管理 API Key。
客户端在请求中携带 Authorization Header，TokenHusk 压缩请求体后原样透传 Header 至目标服务商。

## 2. Golden Rules（最高优先级，绝对不可违背）

### Rule 1: Fail-Open（绝不阻断用户请求）
- **DO**: 任何压缩 Stage 发生 panic、超时、格式解析错误时，必须立即 catch 异常，
  并 **fallback（回退）到原始未压缩的请求** 继续转发。
- **DON'T**: 永远不要因为压缩逻辑的错误而向客户端返回 500 错误，或中断 SSE 流。
  代理的稳定性必须高于压缩功能。

### Rule 2: Strict Latency Budget（严格的延迟预算）
- **DO**: 整个压缩管线（Pipeline）的处理时间硬上限为 **50ms**。
  必须实现全局超时控制（tokio::time::timeout），超时立即放弃压缩，使用原始请求。
- **DON'T**: 不要在请求处理的主线程/异步任务中进行任何阻塞操作
  （如同步文件 I/O、未设超时的网络请求、调用外部 LLM 进行压缩）。

### Rule 3: Zero Semantic Alteration（零语义篡改）
- **DO**: 压缩必须是**确定性的、基于规则的**（如去除 null 值、折叠重复行、截断长数组）。
- **DON'T**: 绝不使用 LLM 来"重写"或"总结"用户的核心 Prompt 意图。
  绝不修改 System Prompt 的核心指令（仅允许在末尾追加简洁性约束）。

### Rule 4: Zero Key Storage（零密钥存储）
- **DO**: TokenHusk 是透明代理。API Key 由客户端持有，随请求的 Authorization Header 到达，
  TokenHusk **原样透传** 该 Header 至目标服务商，不解析、不存储、不记录。
- **DO**: 在 Recorder / Logger / SQLite 中，Authorization Header 必须被强制脱敏为
  `Bearer [REDACTED]`，任何代码路径不得将完整 Key 写入磁盘或日志。
- **DON'T**: 绝不在 SQLite、日志文件、localStorage、配置文件中存储 API Key。
- **DON'T**: 绝不将用户的 Prompt 内容或 Key 发送到除目标 LLM Provider 之外的任何第三方。
- **例外**: 仅当用户**主动、明确**开启"Key 托管（高级模式）"时，
  才可将 Key 存入操作系统原生 Keychain（macOS Keychain / Windows Credential Manager /
  Linux Secret Service），且界面上必须显示醒目的安全提示。此功能为 P2 优先级，MVP 不实现。

### Rule 5: Header Passthrough Integrity（Header 透传完整性）
- **DO**: 除 `Content-Length`（因压缩后 body 变化需重算）和 `Content-Encoding` 外，
  所有原始请求 Header（Authorization、X-Request-ID、User-Agent 等）必须原样透传。
- **DON'T**: 不得添加、删除、修改任何非必要的 Header。不得注入 TokenHusk 标识 Header。

### Rule 6: Emergency Restore（紧急还原）
- **DO**: 必须提供一键还原功能，将所有被 TokenHusk 修改过配置的客户端恢复为原始 base_url。
  还原操作不依赖 TokenHusk 代理运行状态。
- **DO**: 每次配置修改前必须创建 `.bak` 备份文件，备份路径记录在本地 SQLite 中。

## 3. Tech Stack & Environment

- **Frontend**: React 18 + TypeScript + Tailwind CSS + Vite
- **Backend (Core)**: Rust + Tauri 2.0
- **HTTP Proxy**: `axum` + `hyper` + `tokio`（必须支持 SSE 流式透传）
- **Database**: SQLite（via `rusqlite`），仅用于本地观测数据记录
- **Token Counting**: `tiktoken-rs`（OpenAI）+ 自定义 Claude tokenizer 逻辑
- **Code Parsing**: `tree-sitter`（用于代码上下文压缩）
- **Serialization**: `serde` + `serde_json`
- **Key Storage（仅高级模式）**: `keyring-rs` → 系统原生 Keychain

## 4. Architecture & Design Patterns

### 4.1 请求生命周期（方案 B 透明代理）

```text
[AI Client]
  │  POST /v1/chat/completions
  │  Authorization: Bearer sk-xxx  ← 客户端携带
  │  Body: { messages: [...] }
  ▼
[Tauri Proxy (axum, localhost:10520)]
  │
  ├─ 1. 路由匹配：/v1/* → https://api.openai.com
  │     /v1/messages → https://api.anthropic.com
  │
  ├─ 2. 提取 Headers（暂存，不参与压缩）
  │     ⚠️ Authorization 仅在内存中短暂存在，转发后释放
  │
  ├─ 3. Pipeline Orchestrator（≤50ms）
  │     ├── PreCheck（是否值得压缩）
  │     ├── Adapter（Provider ↔ UnifiedRequest）
  │     ├── Stage A: CacheAligner
  │     ├── Stage B: StructureCleaner
  │     ├── Stage C: ContextManager
  │     ├── Stage D: OutputConstraint
  │     └── PostValidate（压缩后 token ≥ 原始 → 回滚）
  │
  ├─ 4. 重建 Body → 重算 Content-Length
  │
  ├─ 5. 透传原始 Headers + 新 Body → 目标服务商
  │     Authorization: Bearer sk-xxx  ← 原样透传
  │
  ├─ 6. SSE 流式回传（不缓冲）
  │
  └─ 7. Recorder（异步，Header 已脱敏）
        → SQLite 写入
```

### 4.2 Pipeline Architecture（管线架构）

请求处理必须遵循严格的管线顺序，每个 Stage 必须是**纯函数**
（输入 `UnifiedRequest`，输出 `UnifiedRequest`，无副作用）：

1. **Adapter Layer**: Provider 特定格式 ↔ `UnifiedRequest` 双向转换
2. **Pre-Check**: 检查是否满足压缩条件（如 token 数是否值得压缩）
3. **Stage A (CacheAligner)**: 调整结构以优化 Provider 的 Prompt Cache 命中率
4. **Stage B (StructureCleaner)**: JSON 压缩、日志去重、工具输出裁剪、代码上下文精简
5. **Stage C (ContextManager)**: 历史消息滑动窗口与重要性裁剪
6. **Stage D (OutputConstraint)**: 注入 max_tokens 和简洁性 System 指令
7. **Post-Validate**: 验证压缩后的 JSON 合法性和 Token 数（如果压缩后 ≥ 原始，则回滚）
8. **Recorder**: 异步记录观测数据（不阻塞主流程，Header 强制脱敏）

### 4.3 Fallback Mechanism（兜底机制）

- 每个 Stage 必须返回 `Result<UnifiedRequest, StageError>`
- Pipeline Orchestrator 负责捕获 `StageError`，记录日志，
  并决定是跳过该 Stage 还是完全回退到原始请求
- **最终兜底**：即使所有 Stage 全部失败，也必须将原始请求原样转发，绝返回 502/500

### 4.4 路由配置模型（替代 Key 配置）

```toml
# tokenhusk.toml — 用户只需配置目标地址，无需 Key
[routes]
"/v1/chat/completions" = { target = "https://api.openai.com" }
"/v1/messages"         = { target = "https://api.anthropic.com" }
"/v1/chat/completions" = { target = "https://api.deepseek.com", match_header = "X-Provider: deepseek" }

[proxy]
listen = "127.0.0.1:10520"
pipeline_timeout_ms = 50
default_mode = "observe"  # observe / conservative / standard / aggressive
```

## 5. Coding Standards & Conventions

### Rust Backend
- **Error Handling**: 使用 `thiserror` 定义业务错误，使用 `anyhow` 处理应用级错误。
  绝不在生产代码中使用 `unwrap()` 或 `expect()`。
- **Async/Await**: 所有 I/O 操作必须是异步的。CPU 密集型任务必须使用
  `tokio::task::spawn_blocking`。
- **SSE Streaming**: 代理转发必须使用 `axum::response::sse` 或底层 `hyper` 流式 body，
  **严禁**将整个 LLM 响应缓冲到内存中再返回。
- **Header 脱敏**: 任何将 Headers 写入日志/DB 的代码路径，必须经过
  `sanitize_headers()` 函数处理。该函数是 `#[must_use]` 的，编译器强制调用。
- **Naming**: PascalCase（类型）/ snake_case（函数/变量）/ SCREAMING_SNAKE_CASE（常量）

### TypeScript Frontend
- **State Management**: React Context + `useReducer` 或 Zustand
- **IPC Communication**: 所有 Tauri IPC 调用封装在自定义 Hooks 中
- **Typing**: 严禁使用 `any`。所有 IPC 数据结构在 `src/types/ipc.ts` 中定义

## 6. Performance & Observability Constraints

- **Memory**: 处理超大 Prompt（>100K tokens）时，避免多次深拷贝 JSON 树
- **Database**: SQLite 写入必须批量/异步执行，避免高频单条插入
- **Logging**: 使用 `tracing` crate。日志中绝不出现完整 Authorization Header
- **Recorder 脱敏规则（硬编码）**:
  ```rust
  // 此函数标记为 #[must_use]，任何 Header 序列化必须经过此函数
  #[must_use]
  pub fn sanitize_headers(headers: &HeaderMap) -> Vec<(String, String)> {
      headers.iter().map(|(k, v)| {
          let val = if k.as_str().eq_ignore_ascii_case("authorization") {
              "Bearer [REDACTED]".to_string()
          } else {
              v.to_str().unwrap_or("[binary]").to_string()
          };
          (k.to_string(), val)
      }).collect()
  }
  ```

## 7. Testing & Quality Assurance

- **Unit Tests**: 所有压缩 Stage 必须有 100% 单元测试覆盖率
- **Integration Tests**: Mock LLM Server 端到端测试 SSE 透传和 Fallback
- **Header Passthrough Test**: 验证 Authorization / X-Request-ID / 自定义 Header
  在压缩前后完全一致（byte-level comparison）
- **Key Leak Test**: 自动化扫描 SQLite 文件和日志文件，
  断言不包含任何匹配 `sk-[a-zA-Z0-9]{20,}` 模式的字符串
- **Golden Tests**: 标准 Prompt 压缩前后质量对比
- **Fuzzing**: JSON 解析和 Token 计数模块模糊测试

## 8. 给 AI 助手的特别提示

在生成任何代码前，请先检查是否违背了上述 Golden Rules。特别注意：
1. 如果代码涉及 Header 处理，必须检查是否经过 `sanitize_headers()`
2. 如果代码涉及 SQLite 写入，必须确认不包含 Authorization 信息
3. 如果代码涉及配置修改，必须包含 `.bak` 备份逻辑
4. 如果用户的需求与 Rule 1 (Fail-Open) 或 Rule 2 (50ms) 冲突，
   请**拒绝直接实现**，指出冲突并提供符合规则的替代方案
5. **绝不生成任何存储、解析、转发 API Key 到非目标地址的代码**
