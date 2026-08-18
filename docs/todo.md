## 🔍 Phase 2.5: 现状审计与假数据清除（建议先执行）

```markdown
# Role
你是一位严格的代码审计工程师，负责找出项目中所有"假实现"并替换为真实实现。

# Context
TokenHusk v2.0 项目已完成 Phase 0 和 Phase 1，但存在严重问题：
- 界面展示的数据是硬编码的 mock 数据
- 按钮和功能是空壳（点击无实际效果）
- 上游服务商地址无法配置
- 统计、请求列表、Diff 查看器可能都是假的

在继续开发新功能之前，必须先彻底清除所有假实现。

请先阅读项目根目录下的 CLAUDE.md 约束文件。

# Task

## Step 1: 全面审计（输出审计报告）
扫描整个项目，找出以下问题并输出清单：

1. **Mock 数据清单**：
   - 搜索所有硬编码的演示数据（如 `mockData`、`fakeStats`、`demoRequests`、
     `sampleData`、`dummyData` 等命名）
   - 搜索所有硬编码的数字统计（如 `savedTokens: 12345`）
   - 搜索所有硬编码的请求列表
   - 列出每个假数据的文件路径、行号、当前值

2. **空壳功能清单**：
   - 找出所有 onClick 处理器为空或只有 console.log 的按钮
   - 找出所有 Tauri command 只有 `todo!()` 或返回固定值的
   - 找出所有 IPC Hook 返回硬编码数据的
   - 列出每个空壳功能的文件路径和函数名

3. **配置系统检查**：
   - 检查是否存在真实的上游地址配置功能
   - 检查代理是否真的读取了配置文件
   - 检查配置是否能持久化

4. **数据链路检查**：
   - 检查 SQLite 表是否已创建
   - 检查 Recorder 是否真的在写入数据
   - 检查前端是否真的在从后端读取数据

## Step 2: 修复清单（按优先级）
基于审计结果，生成修复任务清单，按以下优先级：

### P0（必须立即修复）
1. **上游地址配置功能**：
   - 在设置页面实现真实的上游地址输入框
   - 支持配置多个服务商（名称 + 目标地址）
   - 保存到 `tokenhusk.toml` 或 SQLite（二选一，说明理由）
   - 代理服务器必须读取此配置并真实转发
   - 提供"测试连接"按钮：真实发送一个最小请求到目标地址，验证连通性
   - **禁止**：任何硬编码的默认目标地址

2. **SQLite 数据链路打通**：
   - 确保 Recorder 真实写入每次请求的数据
   - 确保前端从 SQLite 真实读取（通过 Tauri command）
   - 表结构至少包含：
     ```sql
     CREATE TABLE requests (
       id INTEGER PRIMARY KEY,
       timestamp TEXT NOT NULL,
       provider TEXT NOT NULL,
       route TEXT NOT NULL,
       original_tokens INTEGER NOT NULL,
       compressed_tokens INTEGER NOT NULL,
       savings_tokens INTEGER NOT NULL,
       latency_ms INTEGER NOT NULL,
       status TEXT NOT NULL,  -- 'compressed' | 'skipped' | 'fallback'
       skip_reason TEXT,
       original_body TEXT NOT NULL,
       compressed_body TEXT NOT NULL
     );
     ```

3. **仪表盘真实数据**：
   - 今日请求次数：`SELECT COUNT(*) FROM requests WHERE date(timestamp) = date('now')`
   - 节省 Token 数：`SELECT SUM(savings_tokens) FROM requests WHERE ...`
   - 节省比例：计算得出，不是硬编码
   - 最近请求列表：真实从 SQLite 查询
   - **禁止**：任何 `Math.random()` 生成的演示数据

### P1（本阶段内完成）
4. **请求详情 & Diff 查看器**：
   - 点击请求行，从 SQLite 读取 original_body 和 compressed_body
   - 使用真实的 diff 算法（推荐 `diff` npm 包或 `similar` crate）
   - 高亮显示差异部分
   - **禁止**：显示固定的示例文本

5. **代理启停真实控制**：
   - 启动/停止按钮必须真实控制 axum 服务器
   - 托盘状态必须反映真实运行状态
   - **禁止**：按钮只改变 UI 状态而不影响后端

6. **压缩开关真实生效**：
   - 暂停模式下请求必须真实绕过压缩管线
   - 恢复后必须真实走压缩
   - 通过日志或 SQLite 状态字段可验证

## Step 3: 实施修复
按清单逐项修复。每修复一项，必须：
1. 运行相关测试
2. 在界面上实际操作验证
3. 截图或输出验证结果

# Constraints（强制）
- **严禁**使用 `mock`、`fake`、`demo`、`sample`、`dummy`、`placeholder` 等命名的数据
- **严禁**在组件中硬编码统计数字或请求列表
- **严禁**使用 `TODO`、`FIXME`、`// later` 等占位注释代替实现
- **严禁**按钮 onClick 为空或只有 console.log
- 所有数据展示必须能追溯到真实数据源（SQLite / 运行时状态 / 配置文件）
- 所有 Tauri command 必须有真实实现
- 如果某功能确实无法在当前阶段实现，必须在代码中显式 `throw new Error('Not implemented: xxx')`，**不允许静默返回假数据**

# Acceptance Criteria（必须全部满足才算完成）
- [ ] 审计报告已输出，列出所有发现的假数据和空壳功能
- [ ] 上游地址可在设置页面配置，保存后代理真实使用该地址转发
- [ ] 测试连接按钮能真实验证目标地址可达性
- [ ] 通过代理发送真实请求后，SQLite 中能查到对应记录
- [ ] 仪表盘显示的数字与 SQLite 中 `SELECT` 结果一致
- [ ] 最近请求列表与 SQLite 中记录一致
- [ ] 点击请求能看到真实的压缩前后 Diff
- [ ] 暂停模式下新请求的 status 字段为 'skipped'
- [ ] 全项目搜索 `mock|fake|demo|sample|dummy`（不区分大小写）无业务数据命中
- [ ] 全项目搜索 `TODO|FIXME` 无遗留

# 验证脚本要求
请编写一个 `scripts/verify-phase2.5.sh`（或 .ps1），自动化验证：
1. 启动应用
2. 通过代理发送一个真实请求（可用 curl 模拟）
3. 查询 SQLite 确认记录存在
4. 输出验证结果 PASS/FAIL
```

---

## 🟡 Phase 3: 配置向导真实实现（3 周）

```markdown
# Role
你是一位精通跨平台系统集成（macOS/Windows/Linux）的全栈工程师。

# Context
TokenHusk v2.0 Phase 3。Phase 2.5 已清除假数据，核心数据链路已打通。
现在要实现**真实的**配置向导，帮助用户一键接入 TokenHusk。
代理模式：**透明代理**。配置向导**不需要**用户输入 API Key，
只需将客户端的 base_url 指向 TokenHusk。

请先阅读项目根目录下的 CLAUDE.md 约束文件。

# Task

## 1. 应用检测引擎（真实扫描，不允许硬编码列表）

实现 `detect_apps` Tauri command，真实扫描以下位置：

### macOS
- `/Applications/*.app`：ChatBox.app、Cherry Studio.app、Cursor.app、
  LobeChat.app、OpenCat.app 等
- `~/Library/Application Support/` 下的配置目录
- Claude Code：检查 `which claude` 和 `~/.claude/settings.json`

### Windows
- `C:\Program Files\` 和 `%LOCALAPPDATA%\Programs\` 下的已知应用
- 注册表 `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
- `%APPDATA%` 下的配置目录

### Linux
- `/usr/share/applications/*.desktop`
- `~/.config/` 下的配置目录
- `which` 检查 CLI 工具

**输出格式**：
```json
{
  "apps": [
    {
      "name": "ChatBox",
      "level": "A",  // A=自动配置 B=Diff确认 C=教程指引
      "installed": true,
      "config_path": "/Users/xxx/Library/Application Support/ChatBox/config.json",
      "current_base_url": "https://api.openai.com/v1",
      "detected_version": "1.2.3"
    }
  ]
}
```

**禁止**：返回硬编码的假应用列表。如果扫描不到，就返回空数组。

## 2. 配置修改引擎（真实文件操作）

### A 级应用（Codex \ Claude Code）
1. 读取客户端配置文件（JSON/YAML/TOML）
2. **创建 `.bak` 备份**（必须！备份路径记录到 SQLite）
3. 修改 `baseUrl` / `api_url` / `endpoint` 字段为 `http://127.0.0.1:10520`
4. **不修改** API Key 字段
5. 写回文件
6. 验证修改后的文件格式合法

### B 级应用（Cursor）
1. 读取配置文件
2. 生成修改 Diff 展示给用户
3. 用户确认后执行修改
4. 创建备份

### C 级应用（ 其他 CLI）
1. 生成配置教程（复制粘贴命令）
2. 提供一键复制按钮
3. 不直接修改文件

## 3. 连通性测试（真实请求）

配置完成后，向导必须执行真实测试：
1. 让用户确认客户端已填写 API Key（提示文字，不收集 Key）
2. 通过 TokenHusk 代理发送一个最小请求：
   ```json
   {
     "model": "gpt-4o-mini",
     "messages": [{"role": "user", "content": "hi"}],
     "max_tokens": 5
   }
   ```
3. 验证：
   - 代理能收到请求（日志可见）
   - 请求被转发到目标地址
   - 收到流式响应
   - SQLite 中有此测试请求的记录
4. 显示测试结果（成功/失败 + 详细错误）

**禁止**：跳过真实测试直接显示"配置成功"。

## 4. 紧急还原（真实恢复）

1. 从 SQLite 查询所有备份记录
2. 一键恢复：将 `.bak` 文件内容写回原配置文件
3. 恢复后验证文件内容一致
4. 显示恢复结果摘要
5. **不依赖**代理运行状态（即使代理崩溃也能还原）

## 5. 向导 UI

- 步骤条清晰显示当前进度
- 每步都有真实的 loading 状态（对应真实的后台操作）
- 错误提示必须显示真实错误信息（不是"发生错误"这种空话）
- 完成后显示真实摘要（修改了哪些文件、备份在哪里）

# Constraints
- **严禁**返回硬编码的应用列表
- **严禁**跳过真实文件操作（所有"修改配置"必须真实读写文件）
- **严禁**跳过真实连通性测试
- 每次文件修改前必须创建 `.bak` 备份
- 备份路径必须记录到 SQLite（用于紧急还原）
- 向导中**不得出现** API Key 输入框
- 所有错误必须显示真实错误信息

# Acceptance Criteria
- [ ] 在至少一台真实机器上，向导能检测到已安装的 AI 应用
- [ ] 对 A 级应用，向导能真实修改 base_url 并创建备份
- [ ] 修改后客户端能正常通过 TokenHusk 发送请求
- [ ] 连通性测试能真实发送请求并显示结果
- [ ] 紧急还原能真实恢复原始配置
- [ ] 还原后客户端能直连原地址
- [ ] 全项目搜索无 mock 数据
- [ ] 所有 Tauri command 有真实实现
```

---

## 🟠 Phase 4: 压缩引擎补全 + 多服务商真实适配（4 周）

```markdown
# Role
你是一位精通 LLM 上下文管理和多 Provider API 差异的系统架构师。

# Context
TokenHusk v2.0 Phase 4。核心代理、UI、配置向导已真实可用。
现在需要补全所有压缩 Stage 和多服务商适配，达到"标准模式省 30-50%"的目标。
代理模式：**透明代理**。多服务商适配仅涉及 Body 格式转换和路由，
Header（含 Authorization）始终原样透传。

请先阅读项目根目录下的 CLAUDE.md 约束文件。

# Task

## 1. Stage A: CacheAligner（真实实现）

### Anthropic
- 识别 system prompt 中的稳定前缀（不随请求变化的部分）
- 为稳定前缀添加 `cache_control: {"type": "ephemeral"}` 标记
- 将动态字段（时间戳、随机 ID 等）移到消息末尾
- **验证**：压缩后的请求在 Anthropic API 上能真实命中缓存
  （响应中 `usage.cache_read_input_tokens > 0`）

### OpenAI
- 确保前 1024 token 内容稳定
- 动态内容移至消息末尾
- **验证**：连续两次相同请求，第二次 `usage.prompt_tokens_details.cached_tokens > 0`

### DeepSeek / 通义 / Ollama
- 跳过（无 Prompt Caching），但要在日志中明确标记"skip: no cache support"

## 2. Stage B 补全

### ToolOutputTrimmer
- JSON 工具输出 → JsonCrusher
- 文件内容 → 保留首 N 尾 M 行，中间折叠为 `[... X lines omitted ...]`
- 搜索结果 → 保留 top-3
- 命令输出 → 保留首尾 + 所有 ERROR/WARN 行
- **真实测试**：用真实的 Claude Code / Cursor 工具输出样本测试

### CodeContextReducer
- 集成 tree-sitter（Rust binding）
- 支持语言：Python、JavaScript、TypeScript、Rust、Go
- 移除注释和空行
- 折叠 import 块为摘要
- 目标文件保持完整，非目标文件保留函数签名
- **禁止**：删除函数体
- **真实测试**：用真实代码文件测试，压缩后模型仍能理解代码

## 3. Stage C: ContextManager

- SlidingWindow：保留最近 N 轮（默认 10）
- ImportanceScorer（确定性规则）：
  - 包含代码块 +3
  - 包含结论性语句 +2
  - 包含错误信息 +2
  - 包含用户指令 +2
  - 寒暄/确认类 -1
- TokenBudget：按预算从后往前填充
- protect_recent：最近 3 轮永不裁剪
- **真实测试**：构造 20 轮对话，验证裁剪结果符合预期

## 4. 多服务商 Adapter（真实实现）

每个 Adapter 必须：
1. 实现 request Body 双向转换（Provider 格式 ↔ UnifiedRequest）
2. 实现 response Body 解析（提取 usage 字段）
3. 处理 usage 字段缺失的情况（估算或标记 unknown）
4. 通过真实端到端测试（真实调用该 Provider 的 API）

| Provider | 端点 | 测试要求 |
|---|---|---|
| OpenAI | `/v1/chat/completions` | 真实调用，验证流式响应 |
| Anthropic | `/v1/messages` | 真实调用，验证 cache_control |

就这两种provider格式，其他的就是输入服务商对应的地址，这个可以内置多家主流服务商的地址

**禁止**：只实现 OpenAI 然后"假设"其他 Provider 也能工作。

## 5. 路由配置系统

- 支持 `tokenhusk.toml` 中配置多条路由规则
- 支持按路径前缀匹配 + 可选 Header 条件匹配
- 热加载：修改 toml 后无需重启代理（使用 `notify` crate 监听文件变化）
- UI 中提供路由管理界面（增删改查）
- **验证**：修改 toml 后 5 秒内生效

## 6. 策略预设系统

- 观测模式：只记录不压缩
- 保守模式：A + D
- 标准模式（默认）：A + B（保守）+ C + D
- 激进模式：A + B（激进）+ C + D
- 自定义：各 Stage 独立开关 + 参数调节
- 策略切换热加载，无需重启
- UI 中提供策略选择器和参数调节面板

# Constraints
- CacheAligner 绝不能改变语义，只调整结构
- CodeReducer 必须使用 tree-sitter 精确解析，**禁止用正则**
- ImportanceScorer 必须是确定性规则，**禁止调用 LLM**
- 所有 Adapter 必须通过真实端到端测试
- Adapter **不得接触、修改、记录** Authorization Header
- 每个 Stage 必须有完整的单元测试（覆盖率 > 80%）

# Acceptance Criteria
- [ ] Anthropic 请求带正确的 cache_control，真实命中缓存
- [ ] OpenAI 请求第二次真实命中 Prompt Cache
- [ ] ToolOutputTrimmer 对 5K token 工具输出压缩 > 60%（真实样本）
- [ ] CodeReducer 移除注释/import 后代码仍可被模型理解（真实测试）
- [ ] ContextManager 在 20K token 预算下正确裁剪历史
- [ ] 5 个 Provider Adapter 均通过真实端到端测试
- [ ] 路由配置热加载 5 秒内生效
- [ ] 4 个预设模式切换即时生效
- [ ] 标准模式下真实场景节省 30-50%（用真实 Claude Code 会话测试）
- [ ] 全项目无 mock 数据

```

---

## 🔴 Phase 5: 质量守护 + 性能 + 开源发布（3-5 周）

```markdown
# Role
你是一位注重软件质量、用户体验和开源工程化的 Tech Lead。

# Context
TokenHusk v1.0 Phase 5（最终阶段）。所有核心功能已真实可用。
现在需要构建质量守护闭环，确保"压缩不会静默降低回答质量"，
并完成开源发布准备。
代理模式：**透明代理**。所有安全声明围绕"零密钥存储"展开。

请先阅读项目根目录下的 CLAUDE.md 约束文件。

# Task

## 1. 三层质量守护（真实实现）

### Layer 1 自动验证（Pipeline 内同步执行）
- 压缩后 token ≥ 原始 → 跳过压缩
- JSON 格式非法 → 回滚该消息
- 结构不完整（缺少必需字段）→ 回滚
- System Prompt 被修改 → 警告日志
- **验证**：人为构造每种异常情况，确认回滚生效

### Layer 2 用户审查
- Diff 查看器集成 👍/👎 反馈按钮
- 反馈真实写入 SQLite
- 支持"本次跳过"和"此类内容禁用"

### Layer 3 自适应调整
- 连续 3 次 👎 → 自动降低压缩强度一级
- 连续 5 次 👎 → 暂停该 Stage + 托盘通知
- 手动恢复机制
- 状态持久化到 SQLite
- **验证**：模拟连续负反馈，观察自适应行为

## 2. Golden Test 套件

- 准备 10 个标准 Prompt（代码生成/问答/分析/工具调用）
- 压缩前后分别调用模型，对比回答质量
- 评估指标：字符串相似度 + 长度比 + 关键信息保留率
- 质量下降 > 20% → CI 告警
- **必须可在 CI 中离线运行**（mock LLM 响应，注意：这是唯一允许的 mock）

## 3. 安全审计自动化

- **Key Leak Test**：CI 中自动扫描 SQLite 文件、日志文件、配置文件，
  断言不包含 `sk-[a-zA-Z0-9]{20,}` 模式
- **Header Passthrough Test**：对每个 Provider 验证 Authorization Header
  byte-level 一致性
- **Fuzzing**：对 JSON 解析和 Token 计数模块进行模糊测试（cargo-fuzz）
- 所有测试集成到 GitHub Actions CI

## 4. 性能优化与压测

- 大 Prompt（>50K token）场景压测，确保 P99 < 50ms
- SQLite 查询优化（索引、分页、归档）
- 前端虚拟滚动优化（>10K token 不卡顿）
- 内存占用监控（连续运行 1 小时无泄漏）
- 输出性能报告

## 5. 跨平台打包与分发

- macOS: `.dmg` 
- Windows: `.msi` 
- Linux: `.AppImage` 
- Tauri updater 自动更新配置

## 6. 开源发布准备

- **CONTRIBUTING.md** + **CODE_OF_CONDUCT.md**

# Constraints
- 质量守护 Layer 1 必须在 Pipeline 内同步执行，不能增加额外延迟
- Golden Test 必须可在 CI 中离线运行
- 自适应调整的状态需持久化到 SQLite
- 开源代码中**不能包含**任何硬编码的 API Key 或个人信息
- 文档必须中英双语

```

---
