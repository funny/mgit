# MEMORY.md — mgit AI 工程记忆

> 按主题组织的稳定知识库，跨会话持久化。
> 只记录已验证的结论，不记录推测或临时状态。

---

## 项目基本信息

- **项目名**：mgit — Rust 编写的 Git 多仓库管理工具
- **仓库**：github.com/funny/mgit
- **当前版本**：2.0.0-beta.6（beta 阶段）
- **MSRV**：Rust 1.92.0（硬性约束）
- **主分支**：master，开发分支：develop

## Workspace 结构

```
mgit/        # 核心库 (lib)
mgit-cli/    # CLI 入口 (binary: mgit)
mgit-gui/    # GUI 入口 (binary: mgit-gui)
scripts/     # 打包脚本（windows NSIS / macos DMG / linux cross）
tests/gitea-env/  # Docker 集成测试环境
```

## 发布 Tag 规范（已验证 GitHub Actions）

| 优先级 | Workflow | Tag 格式 | 产出 |
|--------|----------|----------|------|
| **最高** | release-setup | `[0-9]*`（如 `2.0.0`） | CLI+GUI 全平台 + 安装包/DMG/DEB |
| 次要 | release-cli | `cli-[0-9]*` | 仅 CLI 压缩包 |
| 次要 | release-gui | `gui-[0-9]*` | 仅 GUI 压缩包 |

release-setup 产出物：
- `.zip`/`.tar.gz` 各平台压缩包
- Windows：`mgit-{version}-setup.exe`（NSIS）
- macOS：`mgit-{version}-universal.dmg`（Universal Binary）
- Linux：`mgit_{version}_amd64.deb`（cargo-deb）

## 关键依赖版本（已锁定）

- egui/eframe/egui_extras：0.33.3
- tokio：1.43（features = ["full"]）
- tracing：0.1.44
- clap：4.5.54
- snafu：0.8.5
- anyhow：1.0.95
- toml：0.8.20 / toml_edit：0.24.0

## GUI 架构要点

- **异步模型**：全局单一 Tokio Runtime（`OnceLock<Runtime>`，`mgit-gui/src/utils/runtime.rs`）
- **渲染模型**：egui 即时模式，`update()` 为同步帧函数
- **事件系统**：`std::thread::spawn` 后台线程 → `event_tx`（mpsc Sender）→ `drain_event_channel()` → `process_events()`
- **禁止**：在 `update()` 或 `GuiApp::new()` 中执行任何 I/O 或 git 操作

## 已修复 Bug

| ID | 描述 | 修复方案 | Commit |
|----|------|----------|--------|
| TD-001 | `GuiApp::new()` 同步阻塞 UI 线程导致 Windows"未响应" | 将 `load_setting()` + `exec_ops(Refresh)` 移至首帧 `update()` 的 `first_frame` 块中执行 | a4cb8d0 |
| TD-002 | `get_repo_states_parallel` 线程数不加限制，HDD 环境 I/O 争用 | `available_parallelism().min(8)` 限制 worker 上限 | b611a5c |

## 已知 Bug / 技术债（待处理）

| ID | 描述 | 位置 |
|----|------|------|
| TD-003 | `AGENTS.md` 记录的依赖版本和 crate 名称已过时 | `AGENTS.md` |

## v2 重构摘要（000aa68 → HEAD，2026-03-17）

从旧结构（`core/` `cli/` `gui/`）迁移到新 workspace 结构，主要变化：
- 引入 tokio 异步运行时（替代 rayon）
- 错误处理改用 snafu（替代 thiserror）
- 日志改用 tracing（统一写入 `~/.mgit/logs/log.txt`）
- GUI 从单文件编辑器重构为 MVVM 事件驱动架构
- 新增 scripts/ 打包脚本（NSIS/DMG/DEB）
- 新增 release-setup.yml 统一发布流程
- 新增 process_guard：Windows Job Object / Linux PR_SET_PDEATHSIG

## 代码风格约定

- 错误处理分层：core→snafu，cli→anyhow，gui→tracing+BackendEvent
- Serde 字段：`#[serde(rename_all = "kebab-case")]`
- 跨平台路径：统一使用 `PathExtension::norm_path()`
- Windows git 调用：`cmd /C git ...` + `CREATE_NO_WINDOW` flag
- 日志格式：`tracing::info!(key = val, "snake_case_event_name")`

## 测试命令

```bash
cargo test                          # 常规测试
cargo test --features=use_gitea    # 集成测试（需 Docker Gitea）
cargo check && cargo clippy && cargo fmt --check  # 提交前检查
```
