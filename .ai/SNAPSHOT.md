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

- 版本：`2.0.0-beta.7`，分支：`master`
- v2 重构已完成（commit `7015e15` 起），共 24 commits，169 文件变动
- 核心变化：tokio 异步、snafu 错误、tracing 日志、MVVM GUI 架构
- 新增统一发布流程：`release-setup.yml`（Tag `[0-9]*`）
- 已知高优先级 bug：TD-001（GUI 启动未响应）、TD-002（HDD 环境 I/O 争用）

**发布 Tag 规范（经 GitHub Actions 核实）**：
- 主发布：纯数字 Tag（`2.0.0`）→ release-setup.yml（最高优先级）
- 次级：`cli-*` / `gui-*` → 独立构建

---

## 2026-07-02 | 2.0.0 正式版本号发布

**操作**：将 workspace package 版本从 `2.0.0-beta.8` 提升到 `2.0.0`

**提交**：最终发布提交

**验证**：
- `cargo check`：通过；保留既有 `mgit-gui/src/utils/system.rs` dead_code warning
- `cargo clippy`：通过；保留既有 clippy warning
- `cargo fmt --check`：通过；已执行 `cargo fmt`
- `cargo test`：未通过；`cli_fetch_simple` 访问 `https://gitee.com` 时无法读取用户名（`Device not configured`）

---

## 2026-07-03 | 2.0.1 CLI 输出修正

**操作**：将 workspace package 版本从 `2.0.0` 提升到 `2.0.1`

**提交**：`f624c27`

**变更**：
- CLI 默认终端 tracing 级别改为 WARN，避免普通命令输出 INFO 日志前缀
- 新增全局 `--verbose`：一次输出 INFO，重复输出 DEBUG
- 保留无 `--verbose` 时的 `RUST_LOG` 高级覆盖能力

**验证**：
- `cargo check`：通过；保留既有 `mgit-gui/src/utils/system.rs` dead_code warning
- `cargo clippy`：通过；保留既有 clippy warning
- `cargo fmt --check`：通过
- `mgit --version`：输出 `mgit 2.0.1`
- 默认无 `INFO`，`--verbose` 可打开 `INFO`

---

## 2026-07-03 | 2.0.2 CLI 展示性输出路由重构

**操作**：将 workspace package 版本从 `2.0.1` 提升到 `2.0.2`

**变更**：
- `Progress` trait 新增 `fn on_message(&self, _message: StyleMessage)`，默认 no-op 实现
- CLI `MultiProgress::on_message` 通过 `multi_progress.println(...)` 输出，避免与 spinner 抢行
- GUI `OpsMessageCollector::on_message` 走 `tracing::info!` 写日志，保持原行为
- ops 中的操作横幅 / 分节标题 / 逐条结果（`ops_start`、`"Track status:"`、`"  + {}"`、`git_new_branch`、`remove_file_succ` 等）从 `tracing::info!` 改走 `progress.on_message(...)`
- 所有 ops 函数签名加 `progress: impl Progress`（含原先无 progress 的 `init`/`snapshot`/`clean`/`log_repos`/`new_branch`/`del_branch`/`new_tag`）
- `MultiProgress::on_batch_finish` 残留的 `tracing::info!("")` 删除
- new-remote-branch / del-remote-branch / new-tag 静默跳过补提示：`branch.is_none()` → `xxx: invalid branch in config file, skipped`；`ignore` 命中 → `xxx: ignored`

**动机**：原 `tracing::info!` 默认 WARN 级别下被过滤，用户看不到操作横幅与逐条结果；且走 stderr、剥离 `StyleMessage` 彩色。改走 `Progress` trait 后 CLI 默认可见、保留彩色、与进度条不冲突，GUI 行为不变。

**验证**：
- `cargo check` / `cargo check --tests`：通过
- `cargo clippy`：无新增 warning（14 个 `double_must_use` + 1 个 `version` 字段 dead_code 均为既有）
- `cargo fmt --check`：通过
- `cargo test -p mgit --lib`：16/16 通过
- 集成测试 `cli_init_simple` 因环境网络（gitee 鉴权）失败，与本次改动无关

---

