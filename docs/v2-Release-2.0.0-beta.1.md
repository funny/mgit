# Release 2.0.0-beta.1

本版本是一个重要的里程碑，完成了架构的全面重构，迁移到了基于 Workspace 的项目结构，并在状态管理和配置持久化方面进行了重大改进。

### Refactor (重构)
- **Project Structure**: 迁移至 Cargo Workspace 模式，将代码库拆分为 `mgit`、`mgit-cli` 和 `mgit-gui` 三个 crate，实现了更好的模块化。
- **Core Architecture**: 重命名了核心配置变量（`toml_config` -> `mgit_config`, `toml_repo` -> `repo_config`），使其与类型语义保持一致。
- **GUI Architecture**: 解耦了 `AppContext` 这个"上帝对象"，引入了 `RepoManager` 和 `SessionManager` 分别负责状态和会话管理。
- **Entry Point**: 将 `app/mod.rs` 移动至 `app.rs`，并将 `ops.rs` 的逻辑合并到 `repo_manager.rs` 中，使入口点更清晰，内聚性更高。
- **Progress Trait**: 重构了 `Progress` trait，采用了更具语义的方法名（例如：`repos_start` -> `on_batch_start`）。
- **Logging System**: 更新了初始化逻辑，以适配新的架构，并改进了 tracing 集成。
- **Shell/Cmd Utils**: 重构了命令执行工具（`cmd.rs`, `shell.rs`），并引入了 `process_guard.rs` 以实现更好的进程管理和错误处理。
- **Shell Trait**: `ShellInteraction` trait 定义在 `mgit/src/utils/shell.rs`，CLI 在 `mgit-cli/src/term/` 目录下实现了终端版本。
- **进程保护**: `mgit/src/utils/process_guard.rs` 实现 Windows Job Object，确保主进程崩溃时子进程自动清理。
- **事件系统**: `mgit-gui/src/app/events.rs` 实现事件总线，支持 GUI 和异步操作的桥接。

### Features (新特性)
- **Version Upgrade**: 项目版本升级至 `2.0.0-beta.1`。
- **Dependency Upgrade**: 将 `egui` 和 `eframe` 升级至 `0.33.3` 版本，`toml_edit` 升级至 `0.24.0`（当前最新版本，文档中的 0.26.x 尚未发布）。
- **Config Persistence**: 实现了基于 TOML 的自定义配置持久化方案，替代了 `eframe` 的默认存储。配置现在保存于 `~/.mgit/settings.toml`。
- **CLI Enhancement**: 移除了 `term_size` 依赖，并统一使用 `console` crate 进行终端输出标准化。

### Fixes (修复)
- **Compilation**: 解决了因大规模重构和模块移动导致的所有编译错误和路径引用问题。
- **Warnings**: 清理了整个代码库中未使用的引用、死代码和其他编译器警告。
