# progress.md — mgit 当前进展追踪

> 轻量任务看板，记录当前 sprint 的工作状态。
> 更新规则：开始任务时标记 `[~]`，完成时标记 `[x]`。

---

## 当前版本：2.0.0-beta.6 · 分支：develop

---

## 进行中

| 状态 | 任务 | 说明 |
|------|------|------|
| `[~]` | 分析 GUI 在 Windows 10 HDD 环境未响应问题 | 已定位 TD-001（new() 阻塞）和 TD-002（线程数未限制） |

---

## 待处理

| 优先级 | 任务 | 关联 |
|--------|------|------|
| 高 | 修复 TD-001：将 `exec_ops(Refresh)` 移出 `GuiApp::new()` UI 线程 | `app.rs:104-109` |
| 中 | 修复 TD-002：限制 `get_repo_states_parallel` worker 数量上限 | `repo_manager.rs:767` |
| 低 | 更新/删除过时的 `AGENTS.md` | `AGENTS.md` |

---

## 已完成（本周期）

| 完成时间 | 任务 |
|----------|------|
| 2026-03-17 | v2 重构完成（7015e15）：workspace 重组、async tokio、egui 0.33 |
| 2026-03-17 | 全局 Tokio Runtime 单例（fc18a0a） |
| 2026-03-17 | GUI 新增 Unignore All / Ignore All（444aa3e） |
| 2026-03-17 | 支持拖拽配置文件（1f8148b） |
| 2026-03-17 | 修复 track 命令错误被吞（326b5c7） |
| 2026-03-17 | 修复 new-tag 推送错误仓库路径（07b769f） |
| 2026-03-17 | 修复 GUI 默认分支字段传递错误（7e4ccba） |
| 2026-03-17 | 初始化 AI 工程规范（AGENT.md、CONTRIBUTING.md、.ai/） |

---

## 技术债追踪

| 编号 | 描述 | 严重度 | 状态 |
|------|------|--------|------|
| TD-001 | `GuiApp::new()` 同步阻塞 UI 线程 | 高 | 待修复 |
| TD-002 | 并发 worker 数未限制，HDD 环境 I/O 争用 | 中 | 待修复 |
| TD-003 | `AGENTS.md` 内容过时 | 低 | 待处理 |
