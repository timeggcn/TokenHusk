规范：
1. 阅读AGENTS.md理解本项目的核心原则和架构要求
2. 代码开发规范遵循插件：karpathy-guidelines
3. 生产代码时需要遵循插件：code-simplifier
4. 如果是开发前端页面，样式风格需要遵循插件：ui-ux-pro-max，且需要按照当前项目框架内进行，如需额外组件，需要跟我确认
5. 先检查需求内容有没有错误前提、逻辑跳跃、信息缺失。区分事实、推测和主观观点
6. 开发之前请确保已经完全理解我的需求，可以询问我，在没得到我口头确认前不能进行生成
7. 已经手动改动过的内容，没有我的允许不能擅自修改
需求：
# Role
你是一位全栈开发者，精通 React + Tailwind + Tauri IPC + 跨平台系统集成。

# Context
TokenHusk v1.0 Phase 2。后端代理和压缩管线已就绪。
现在需要构建用户界面和配置助手，让用户能"看到价值"并"安全接入"。
代理模式：**透明代理**。配置助手不需要用户输入 API Key，
只需将客户端的 base_url 指向 TokenHusk，Key 由客户端自行管理。
核心价值感知：L1 观测（知道钱花在哪）+ L2 治理（自动省 40%）。

请先阅读项目根目录下的 AGENTS.md 约束文件。

# Task
请实现以下前端和系统集成模块：

1. **仪表盘 (Dashboard)**
   - 今日概览卡片：请求次数、节省 Token 数、节省比例、预估节省金额
   - Token 消耗分布饼图（工具输出/代码/对话/指令）
   - 节省来源分析柱状图（各 Stage 贡献占比）
   - 最近请求列表（时间、来源 App、服务商、压缩率、状态标签）

2. **请求详情 & Diff 查看器**
   - 点击请求展开详情面板
   - 左右分栏 Diff 视图：左侧原始消息，右侧压缩后消息
   - 高亮显示被移除/修改的内容
   - 质量反馈按钮（👍/👎）
   - **注意**：Diff 视图中不得显示 Authorization Header 内容

3. **配置向导 (Setup Wizard) — 方案 B 简化版**
   - Step 1: 扫描已安装 AI 应用（检测常见路径/环境变量/进程）
   - Step 2: 选择服务商 + 确认目标地址（不需要输入 API Key）
     - 显示提示："你的 API Key 保持在客户端中，TokenHusk 不会接触它"
   - Step 3: 选择压缩预设（观测/保守/标准/激进）
   - Step 4: 执行配置
     - 备份客户端原始配置文件（.bak）
     - 修改客户端 base_url → http://127.0.0.1:10520
     - **不修改客户端的 API Key 字段**
     - 发送测试请求验证连通性
     - 显示完成摘要
   - A 级应用（ChatBox/Cherry Studio）：自动修改
   - B 级应用（Cursor）：显示 Diff 确认
   - C 级应用（Claude Code）：显示教程 + 复制地址

4. **系统托盘 & 安全操作**
   - 托盘菜单：运行状态、今日节省、暂停/恢复、打开仪表盘、紧急还原、退出
   - **紧急还原**：一键恢复所有客户端的原始 base_url（从 .bak 恢复）
     - 不依赖代理运行状态
     - 还原后显示确认对话框
   - 暂停模式：所有请求直接透传，不走压缩管线

5. **Tauri IPC 命令**
   - get_stats / get_recent_requests / get_request_detail
   - start_proxy / stop_proxy / pause_proxy
   - detect_apps / configure_app / emergency_restore
   - submit_feedback
   - **注意**：所有 IPC 返回值中不得包含 Authorization 信息

# Constraints
- UI 必须响应式，适配不同窗口大小
- **界面上不得出现任何 API Key 输入框**（方案 B 核心原则）
- 配置修改前必须创建 .bak 备份文件
- Diff 查看器需支持大文本虚拟滚动（>10K token 不卡顿）
- 所有 IPC 命令需有错误处理和 loading 状态

# Acceptance Criteria
- [ ] 仪表盘实时展示代理统计数据
- [ ] Diff 查看器能清晰展示压缩前后差异（无 Key 泄露）
- [ ] 配置向导全程无 API Key 输入步骤
- [ ] 配置向导能正确检测并配置至少一个 A 级应用（仅修改 base_url）
- [ ] 紧急还原能恢复原始 base_url
- [ ] 托盘暂停后请求直连，恢复后走代理
- [ ] 配置向导中有明确的"零密钥存储"安全提示
