# TokenHusk 🍊

> **帮你把 Prompt 里的垃圾扔掉，而不是帮你写更好的 Prompt。**

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-black.svg)](https://www.rust-lang.org)
[![Website](https://img.shields.io/badge/Website-pfctools.com-green.svg)](https://pfctools.com)

TokenHusk 是一个基于 Tauri 2.0 构建的 **AI Agent 成本控制层桌面应用**。
它通过在本地运行高性能透明代理，拦截发往 LLM 的请求，使用**确定性规则引擎**自动压缩
Prompt 中的冗余信息（巨大的 JSON 工具输出、重复日志、无关代码上下文），
从而显著降低 Token 消耗和响应延迟。

🌐 **官方网站**: [https://pfctools.com](https://pfctools.com)

---

## 🔒 零密钥存储 — 我们碰不到你的 Key

TokenHusk 是**透明代理**。你的 API Key 始终由客户端持有，
TokenHusk 仅压缩请求体，Authorization Header **原样透传**至目标服务商。

- ❌ 不存储 API Key
- ❌ 不记录 API Key
- ❌ 不解析 API Key
- ❌ 不将 Key 发送到任何第三方
- ✅ Key 仅在你的客户端 ↔ 目标 API 之间流动

> 我们不帮你管 Key，我们只帮你省钱。

---

## ✨ 核心特性

| 特性 | 说明 |
|:---|:---|
| 🛡️ **Fail-Open 安全兜底** | 任何压缩异常自动回退原始请求，**绝不阻断**对话 |
| ⚡ **<50ms 极致低延迟** | 纯 Rust 异步管线，P99 < 50ms，体感零延迟 |
| 🎯 **确定性压缩** | 规则引擎驱动，不用 LLM 重写 Prompt |
| 🔌 **即插即用** | 只需改客户端的 base_url，Key 照填，零迁移 |
| 📊 **全链路观测** | 实时仪表盘 + 请求级 Diff 对比 + 节省统计 |
| 🔐 **零密钥存储** | 透明代理模式，Key 不经过 TokenHusk 持久化 |
| 🔄 **一键紧急还原** | 托盘菜单一键恢复所有客户端原始配置 |

## 🏗️ 架构概览

```text
[AI Client]  ──POST + Authorization──▶  [TokenHusk Proxy :10520]
                                            │
                                            ├─ 提取 Headers（暂存，不记录）
                                            ├─ Pipeline（≤50ms）
                                            │   ├─ Stage A: CacheAligner
                                            │   ├─ Stage B: StructureCleaner
                                            │   ├─ Stage C: ContextManager
                                            │   └─ Stage D: OutputConstraint
                                            ├─ 重建 Body + 重算 Content-Length
                                            │
                                            ▼
                                    [LLM Provider API]
                                    Authorization 原样透传 ✅
```

## 🚀 快速开始

### 30 秒接入（以 ChatBox + OpenAI 为例）

```text
Before:  ChatBox → https://api.openai.com/v1  (你的 Key)
After:   ChatBox → http://127.0.0.1:10520/v1  (你的 Key，不变)
```

1. 下载并启动 TokenHusk
2. 打开 ChatBox 设置，将 API 地址改为 `http://127.0.0.1:10520`
3. **API Key 不用动**，照填
4. 完成。TokenHusk 开始观测和压缩

> 💡 也可以使用内置的 **Setup Wizard**，自动检测已安装的 AI 应用并一键配置。

### 从源码构建
rust >= 1.95.0
node >= 18.20.8
```bash
git clone https://github.com/timeggcn/TokenHusk.git
cd TokenHusk
pnpm install
pnpm tauri dev      # 开发模式
pnpm tauri build    # 生产构建
```

## 📦 支持的 LLM 服务商

| 服务商 | 协议 | Cache 优化 | 状态 |
|:---|:---|:---|:---|
| OpenAI | Chat Completions | ✅ Prefix Stability | ✅ |
| Anthropic | Messages API | ✅ cache_control | ✅ |
| DeepSeek | OpenAI Compatible | ⏭️ N/A | ✅ |
| 通义千问 | OpenAI Compatible | ⏭️ N/A | ✅ |
| Ollama | OpenAI Compatible | ⏭️ N/A | ✅ |
| 任意 OpenAI 兼容 | Custom Endpoint | ⚙️ 可配置 | ✅ |

## 🎛️ 压缩预设

| 模式 | 启用的 Stage | 预期节省 | 适用场景 |
|:---|:---|:---|:---|
| 🟢 保守模式 | A + D | 10-20% | 对质量极度敏感 |
| 🟡 标准模式 | A + B + C + D | 30-50% | **推荐日常使用** |
| 🔴 激进模式 | A + B(激进) + C + D | 50-70% | 大量工具调用场景 |

## 🧪 三层质量守护

1. **自动验证**: 压缩后 token ≥ 原始 / JSON 非法 / 结构不完整 → 自动回滚
2. **用户审查**: 每个请求可查看压缩 Diff + 👍👎 反馈
3. **自适应降级**: 连续 3 次 👎 → 自动降低压缩强度；连续 5 次 → 暂停该 Stage

## 📸 截图

> _（此处放置仪表盘截图、Diff 查看器截图、配置向导截图）_

## 🤝 贡献

提交 PR 前请确保：
- [ ] 所有压缩 Stage 有完整单元测试
- [ ] Header 透传测试通过（byte-level）
- [ ] Key Leak Test 通过（日志/DB 无 `sk-*` 泄露）
- [ ] Golden Test 套件通过

## 📄 License

[Apache License 2.0](LICENSE)

## 🔗 链接

| 资源 | 地址 |
|:---|:---|
| 🌐 官网 | [https://pfctools.com](https://pfctools.com) |
| 📖 文档 | [https://pfctools.com/docs](https://pfctools.com/docs) |
| 🐛 Issues | [GitHub Issues](../../issues) |
| 💬 讨论 | [GitHub Discussions](../../discussions) |

---

<div align="center">
  <sub>Built with ❤️ by PFC Tools · <a href="https://pfctools.com">pfctools.com</a></sub>
</div>
```
