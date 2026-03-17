# SNAPSHOT.md — mgit 工程快照记录

> 按时间顺序记录重大里程碑，每条记录不删改，只追加。

---

## 2026-03-17 | 工程已初始化

**操作**：按 2026 AI 原生规范初始化工程结构

**创建文件**：
- `AGENT.md`：AI 协作宪法（环境约束、架构约定、行为准则）
- `CONTRIBUTING.md`：贡献协议（分支策略、commit 规范、发布流程）
- `progress.md`：当前进展追踪看板
- `.ai/MEMORY.md`：跨会话 AI 记忆（主题化知识库）
- `.ai/SNAPSHOT.md`：本文件（里程碑记录）
- `docs/plan/`：AI 规划任务目录

**工程现状快照（基于代码审查）**：

- 版本：`2.0.0-beta.6`，分支：`master`
- v2 重构已完成（commit `7015e15` 起），共 24 commits，169 文件变动
- 核心变化：tokio 异步、snafu 错误、tracing 日志、MVVM GUI 架构
- 新增统一发布流程：`release-setup.yml`（Tag `[0-9]*`）
- 已知高优先级 bug：TD-001（GUI 启动未响应）、TD-002（HDD 环境 I/O 争用）

**发布 Tag 规范（经 GitHub Actions 核实）**：
- 主发布：纯数字 Tag（`2.0.0`）→ release-setup.yml（最高优先级）
- 次级：`cli-*` / `gui-*` → 独立构建

---
