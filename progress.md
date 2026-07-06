# progress.md — mgit 当前进展追踪

> 轻量任务看板，记录当前 sprint 的工作状态。
> 更新规则：开始任务时标记 `[~]`，完成时标记 `[x]`。

---

## 当前版本：2.0.3-beta.1 · 分支：issues/116-self-update

---

## 进行中

无

---

## 待处理

无

---

## 已完成（本周期）

| 完成时间 | 任务 |
|----------|------|
| 2026-07-06 | mgit upgrade 新增 --pre 支持预发布版本、--target-version 指定目标版本 |
| 2026-07-06 | 提取 upgrade 公共逻辑到 mgit 库，GUI Help 菜单新增 Check for Updates |
| 2026-07-06 | 实现 mgit upgrade 自升级命令（87aa764）：从 GitHub Releases 下载匹配本机 target 的资产并原地替换，支持 --force |
| 2026-03-17 | v2 重构完成（7015e15）：workspace 重组、async tokio、egui 0.33 |
| 2026-03-17 | 全局 Tokio Runtime 单例（fc18a0a） |
| 2026-03-17 | GUI 新增 Unignore All / Ignore All（444aa3e） |
| 2026-03-17 | 支持拖拽配置文件（1f8148b） |
| 2026-03-17 | 修复 track 命令错误被吞（326b5c7） |
| 2026-03-17 | 修复 new-tag 推送错误仓库路径（07b769f） |
| 2026-03-17 | 修复 GUI 默认分支字段传递错误（7e4ccba） |
| 2026-03-17 | 初始化 AI 工程规范（AGENT.md、CONTRIBUTING.md、.ai/） |
| 2026-03-17 | 修复 TD-001：GUI 启动未响应（a4cb8d0） |
| 2026-03-17 | 修复 TD-002：worker 数量上限为 8（b611a5c） |
| 2026-03-17 | 修复 CLI 错误输出 Debug 格式，fetch/sync/track/clean 改为可读文本 |
| 2026-05-27 | 修复 new-remote-branch / del-remote-branch / new-tag 错误消息仍用 `{:?}` 的遗漏 |
| 2026-03-17 | 修复 GUI 白屏未响应：check_git_valid 移至后台线程，不再阻塞 new() |
| 2026-03-17 | 增加启动链路结构化日志：new/first_frame/git_check/load_setting/refresh/worker 全链路计时 |
| 2026-03-17 | 增强调试日志：runtime_init/eframe_run_native_start 时间戳、update() 帧间隔+耗时阻塞检测、每条 git 命令计时 |
| 2026-03-17 | 修复 TD-004：WGL SwapBuffers 首帧冻结 — check_git_valid 回归 new() 同步执行，为 GPU 驱动预热 |
| 2026-07-02 | 执行 `cargo fmt` 格式化既有 Rust 源文件（c7576bb） |
| 2026-07-02 | 发布 2.0.0：workspace 版本从 2.0.0-beta.8 提升到 2.0.0（最终发布提交） |
| 2026-07-03 | CLI 增加全局 `--no-color`，默认保留彩色输出，并避免 tracing 字段输出 ANSI 转义串；新增 `--verbose` 控制 INFO/DEBUG 输出（7948a89） |
| 2026-07-03 | 发布 2.0.1：workspace 版本从 2.0.0 提升到 2.0.1（f624c27） |
| 2026-07-03 | 强化 AGENTS 提交规范：提交前必须读取 `CONTRIBUTING.md`，并使用 `<type> 中文描述` 格式（5118b9a） |
| 2026-07-03 | CLI 展示性输出路由重构：`Progress` trait 新增 `on_message`，ops 中的操作横幅/分节标题/逐条结果从 `tracing::info!` 改走 `MultiProgress::println`，默认可见且保留彩色；GUI 侧 `OpsMessageCollector` 写日志保持原行为 |
| 2026-07-03 | 删除 `MultiProgress::on_batch_finish` 中残留的空 `tracing::info!("")` |
| 2026-07-03 | new-remote-branch / del-remote-branch / new-tag 静默跳过补提示：`branch.is_none()` 输出 `xxx: invalid branch in config file, skipped`；`ignore` 命中输出 `xxx: ignored` |
| 2026-07-03 | 发布 2.0.2：workspace 版本从 2.0.1 提升到 2.0.2 |

---

## 技术债追踪

| 编号 | 描述 | 严重度 | 状态 |
|------|------|--------|------|
| TD-001 | `GuiApp::new()` 同步阻塞 UI 线程 | 高 | **已修复** a4cb8d0 |
| TD-002 | 并发 worker 数未限制，HDD 环境 I/O 争用 | 中 | **已修复** b611a5c |
| TD-003 | `AGENTS.md` 内容过时 | 低 | **已修复** b8c13c6 |
| TD-004 | WGL `SwapBuffers()` 首帧冻结白屏（Windows HDD + 独显）| 高 | **已修复** — `check_git_valid` 回归 `new()` 同步执行预热 GPU |
