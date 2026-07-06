# AGENTS.md — mgit 项目 AI 协作宪法

> 本文件是所有 AI Agent 在此工程中工作的最高行为准则。

---

## ⚠️ 每次任务完成后，提交前必须执行

> **不更新以下文件 = 不完整交付。无论任务大小，无需用户提醒。**
> 提交信息规范以 `CONTRIBUTING.md` 为准；AI Agent 每次提交前必须重新读取该文件。

### `progress.md`
- 完成的任务 → 移入"已完成"，附 commit hash
- 技术债已修复 → 状态改为 `已修复 {commit}`

### `.ai/MEMORY.md`
- 已修复的 bug → 从"待处理"移入"已修复"，记录方案和 commit
- 本次验证的稳定结论 → 补充到对应主题下

### `.ai/SNAPSHOT.md`
- 重大里程碑（版本发布、重要重构）→ 追加记录

---

## 1. 项目概览

**mgit** 是一个用 Rust 编写的 Git **多仓库管理工具**，提供 CLI 和 GUI 两种使用方式。

| 属性 | 值 |
|------|----|
| 仓库 | github.com/funny/mgit |
| 当前版本 | 2.0.0-beta.8 |
| 主分支 | `master` |
| 语言 | Rust 2021 edition |
| MSRV | **1.92.0**（硬性约束，不得降低） |

### Workspace 结构

```
mgit/
├── mgit/           # 核心库 (lib crate)：git ops、config、error、utils
├── mgit-cli/       # CLI 入口 (binary: mgit)，clap 4
├── mgit-gui/       # GUI 入口 (binary: mgit-gui)，egui 0.33
├── scripts/
│   ├── windows/    # NSIS 安装包脚本
│   ├── macos/      # DMG 打包脚本
│   └── linux/      # 交叉编译辅助脚本
├── tests/
│   └── gitea-env/  # Docker Gitea 集成测试环境
├── docs/
│   └── plan/       # AI 规划任务文档
├── .ai/            # AI 记忆与快照
├── AGENT.md        # 本文件（宪法）
├── CONTRIBUTING.md # 贡献协议
└── progress.md     # 当前进展追踪
```

---

## 2. 环境约束（硬性规则）

### 2.1 工具链

```toml
rust-version = "1.92.0"   # MSRV，cargo check 会验证
edition = "2021"
resolver = "2"
```

- 禁止使用 `1.92.0` 以下版本才有的 unstable feature。
- 所有依赖统一在 **workspace root `Cargo.toml`** 的 `[workspace.dependencies]` 中声明，子 crate 通过 `{ workspace = true }` 继承。

### 2.2 关键依赖版本（当前锁定）

| 类别 | 包 | 版本 |
|------|-----|------|
| GUI | egui / eframe / egui_extras | **0.33.3** |
| 异步 | tokio | **1.43**（features = ["full"]） |
| 日志 | tracing / tracing-subscriber / tracing-appender | **0.1.44 / 0.3.19 / 0.2.3** |
| CLI | clap | **4.5.54**（derive + cargo + env） |
| 错误 | snafu | **0.8.5**（backtrace） |
| 错误 | anyhow | **1.0.95** |
| 配置 | toml / toml_edit | **0.8.20 / 0.24.0** |
| 序列化 | serde | **1.0.228**（derive） |
| 图像 | image / rfd | **0.25.9 / 0.17.2** |

> 修改依赖版本前需在 `progress.md` 中记录理由。

### 2.3 异步模型

- **全局单一 Tokio Runtime**：通过 `OnceLock<Runtime>` 在 `mgit-gui/src/utils/runtime.rs` 初始化。
- **禁止**在已有 async 上下文内部调用 `block_on`（会 panic）。
- GUI 的 `update()` 帧函数是**同步**的，所有耗时操作必须通过 `std::thread::spawn` + `event_tx` 通道回传，**严禁**在 `update()` 内执行任何 I/O 或 git 操作。
- 后台并发 worker 数量应**显式限制**（建议 ≤ 8），避免在机械硬盘环境因线程过多造成 I/O 争用（已知问题 TD-002）。

### 2.4 平台目标

| 平台 | 架构 | 备注 |
|------|------|------|
| Windows | x86_64-pc-windows-msvc | 产出 .zip + NSIS 安装包 |
| macOS | x86_64-apple-darwin + aarch64-apple-darwin | 产出 Universal DMG |
| Linux (musl) | x86_64 + aarch64 | CLI only（musl 静态链接） |
| Linux (gnu) | x86_64 | CLI + GUI，用于 DEB 包 |

- Windows 下 git 命令通过 `cmd /C git ...` 调用，必须加 `CREATE_NO_WINDOW` flag 避免弹出控制台。
- 路径处理统一使用 `PathExtension::norm_path()` 保证跨平台一致性。

### 2.5 发布流程

#### 主发布（优先级最高）

**release-setup**：Tag 格式 `[0-9]*`（如 `2.0.0`、`2.0.0-beta.7`）

同时构建 CLI + GUI，产出完整发布物：
- 各平台 `.zip` / `.tar.gz` 压缩包（CLI + GUI）
- Windows：NSIS 安装包 `mgit-{version}-setup.exe`
- macOS：Universal Binary DMG `mgit-{version}-universal.dmg`
- Linux：DEB 包 `mgit_{version}_amd64.deb`

#### 次级发布（独立构建）

| Workflow | Tag 格式 | 产出 |
|----------|----------|------|
| release-cli | `cli-[0-9]*`（如 `cli-2.0.0`） | 仅 mgit-cli 各平台压缩包 |
| release-gui | `gui-[0-9]*`（如 `gui-2.0.0`） | 仅 mgit-gui 各平台压缩包 |

---

## 3. 代码规范

### 3.1 命名约定

| 元素 | 规范 | 示例 |
|------|------|------|
| Struct | PascalCase | `RepoState`, `SyncOptions` |
| Function | snake_case | `sync_repo`, `load_config` |
| Constant | SCREAMING_SNAKE_CASE | `DEFAULT_WIDTH`, `GIT_VERSION` |
| Enum | PascalCase | `StateType`, `CommandType` |
| Module | snake_case | `repo_manager`, `session_manager` |
| Serde 字段 | kebab-case | `#[serde(rename_all = "kebab-case")]` |

### 3.2 错误处理分层

```
mgit (core)  → snafu 自定义错误类型（结构化上下文 + Backtrace）
mgit-cli     → anyhow 顶层捕获 + snafu context
mgit-gui     → tracing::error! 记录 + 通过 BackendEvent 回传 UI
```

- 不允许 `unwrap()` 出现在非测试代码中（除非有明确 SAFETY 注释）。

### 3.3 Import 顺序

```rust
// 1. std
use std::path::{Path, PathBuf};
// 2. 第三方
use anyhow::Context;
use tokio::runtime::Runtime;
// 3. 内部
use crate::config::MgitConfig;
use super::events::Event;
```

### 3.4 配置文件格式

- 仓库配置：`.gitrepos`（TOML，kebab-case 字段）
- 用户设置：`~/.mgit/`（TomlUserSettings）
- 项目设置：`~/.mgit/tmp/`（TomlProjectSettings，按路径哈希命名）
- 日志：`~/.mgit/logs/log.txt`

### 3.5 日志规范（tracing 结构化）

```rust
tracing::info!(run_id, repo = local.as_str(), "operation_started");
tracing::error!(error = %e, "operation_failed");
tracing::debug!(duration_ms = elapsed.as_millis(), "event_processed");
```

---

## 4. GUI 架构约定

### 4.1 事件驱动模型（MVVM + mpsc 通道）

```
UI Thread (egui update loop)
  ├── top_view / content_view / handle_windows   ← 纯渲染
  ├── drain_event_channel()                       ← 收集后台事件
  └── process_events() → handle_event()           ← 分发处理

Backend Thread (std::thread::spawn)
  └── event_tx.send(Event::Backend(...))          ← 发送结果回 UI
```

### 4.2 禁止行为

- **禁止**在 `GuiApp::new()` 中执行耗时同步 I/O（阻塞窗口创建 → Windows "未响应"，已知问题 TD-001）。
- **禁止**在 `update()` 帧函数中调用 git 操作或文件读写。
- **禁止**在后台线程直接修改 `AppContext`（通过事件通道回传）。

---

## 5. AI Agent 行为准则

### 5.1 变更范围控制

- **只做被要求的事**，不引入额外重构或功能。
- 修改现有文件前必须先 Read，理解上下文后再 Edit。
- 不创建冗余文件，优先 Edit 现有文件。

### 5.2 安全红线

- **不得**在未经确认的情况下强制覆盖 `master` 分支历史。
- **不得**提交含密钥、Token、密码的文件。
- **不得**执行 `cargo clean`。
- **不得**使用 `--no-verify` 跳过 pre-commit hook。
- **不得**强推远端（`--force` push）。

### 5.3 测试要求

```bash
# 每次代码变更后至少执行：
cargo check              # 确保编译通过
cargo clippy             # 无新增 warning
cargo fmt --check        # 格式合规

# 涉及 mgit core 的变更额外执行：
cargo test

# 涉及网络操作的变更（需 Docker 环境）：
cargo test --features=use_gitea
```

### 5.4 记忆更新义务

见文件顶部"每次任务完成后必须执行"章节，此为强制规则。

### 5.5 Commit 规范（硬性规则）

- **每次提交前必须先读取 `CONTRIBUTING.md`**，并以其中的 Commit 规范为准。
- 提交信息必须使用 `CONTRIBUTING.md` 定义的格式：

```text
<type> 中文描述
```

- 允许的 `type` 仅限：
  - `<feature>`
  - `<fix>`
  - `<refactor>`
  - `<misc>`
  - `<docs>`
  - `<test>`
- 禁止使用 Conventional Commit 格式，例如：
  - `feat(cli): ...`
  - `fix: ...`
  - `docs: ...`
- 提交描述必须使用中文，且应简洁说明本次提交实际内容。
- 如果修改、重写、合并、拆分提交导致 commit hash 变化，必须同步更新：
  - `progress.md`
  - `.ai/MEMORY.md`
  - `.ai/SNAPSHOT.md`（如涉及版本发布或重大里程碑）
- AI Agent 在执行 `git commit` 前必须确认：
  - `git status --short` 中只包含本次任务相关文件
  - commit message 符合 `CONTRIBUTING.md`
  - 不使用 `--no-verify`

---

## 6. 已知技术债

| 编号 | 问题描述 | 严重度 | 相关位置 |
|------|----------|--------|----------|
| TD-001 | `GuiApp::new()` 同步调用 `load_config` 阻塞 UI 线程，导致 Windows "未响应" | 高 | `mgit-gui/src/app.rs:104-109` |
| TD-002 | `get_repo_states_parallel` 线程数未限制（HDD 环境 I/O 争用） | 中 | `mgit-gui/src/app/repo_manager.rs:767` |
| TD-003 | `AGENTS.md` 内容已过时（依赖版本、crate 名称均有误） | 低 | `AGENTS.md` |

---

*本文件由 AI Agent 在 2026-03-17 根据项目代码与 GitHub Actions 自动生成，需人工审阅确认。*
