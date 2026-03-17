# CONTRIBUTING.md — mgit 贡献协议

> 所有向 mgit 提交代码的贡献者（包括 AI Agent）均须遵守本协议。

---

## 1. 分支策略

| 分支 | 用途 | 规则 |
|------|------|------|
| `master` | 主开发分支，包含所有功能 | 功能分支合并目标，日常工作基准 |
| `feat/*` | 新功能开发 | 从 `master` 切出，合并回 `master` |
| `fix/*` | Bug 修复 | 从 `master` 切出，合并回 `master` |
| `refactor/*` | 重构 | 从 `master` 切出，合并回 `master` |
| `docs/*` | 文档更新 | 从 `master` 切出，合并回 `master` |

---

## 2. Commit 规范

格式：`<type> <简短描述>`

| type | 含义 |
|------|------|
| `<feature>` | 新功能 |
| `<fix>` | Bug 修复 |
| `<refactor>` | 重构（不改变外部行为） |
| `<misc>` | 杂项（版本号更新、格式化、配置调整等） |
| `<docs>` | 文档 |
| `<test>` | 测试相关 |

示例（参考历史 commit 风格）：
```
<fix> 修复 GUI 默认分支字段传递错误
<feature> 支持定位配置文件，支持拖拽配置文件
<refactor> 提取 serialize_config 函数并添加为 MgitConfig 方法
<misc> 2.0.0-beta.6
```

---

## 3. 版本号规范

遵循 [SemVer](https://semver.org/)，当前处于 beta 阶段：`2.0.0-beta.x`

版本号统一在 workspace root `Cargo.toml` 的 `[workspace.package]` 中维护：
```toml
[workspace.package]
version = "2.0.0-beta.6"
```

---

## 4. 发布流程

### 主发布（推荐，优先使用）

推送纯数字版本 Tag，触发 `release-setup.yml`，同时构建 CLI + GUI 全平台产物：

```bash
git tag 2.0.0
git push origin 2.0.0
```

产出物：
- `mgit-cli-{version}-{target}.zip/.tar.gz`
- `mgit-gui-{version}-{target}.zip/.tar.gz`
- `mgit-{version}-setup.exe`（Windows 安装包，NSIS）
- `mgit-{version}-universal.dmg`（macOS 通用包）
- `mgit_{version}_amd64.deb`（Linux DEB）

### 次级发布（独立构建，按需使用）

```bash
# 仅发布 CLI
git tag cli-2.0.0 && git push origin cli-2.0.0

# 仅发布 GUI
git tag gui-2.0.0 && git push origin gui-2.0.0
```

---

## 5. 代码提交检查清单

提交前确认：

- [ ] `cargo check` 通过
- [ ] `cargo clippy` 无新增 warning
- [ ] `cargo fmt` 已格式化
- [ ] `cargo test` 通过（涉及 core 变更时）
- [ ] 无 `unwrap()` 添加到非测试代码
- [ ] 无敏感信息（密钥、Token）提交
- [ ] `progress.md` 已更新任务状态

---

## 6. Pull Request 规范

- PR 标题与 commit 类型一致
- 描述中说明：**改了什么**、**为什么改**、**如何测试**
- 必须至少一名 Reviewer 批准后才能合并
- 合并方式：`Squash and Merge`（保持 master 历史清洁）

---

## 7. 测试环境

### 常规测试
```bash
cargo test
```

### 集成测试（需 Docker + Gitea）
```bash
# 启动本地 Gitea 服务
tests/gitea-env/start_gitea.sh   # Linux/macOS
# Windows 请使用 WSL2

# 运行集成测试
cargo test --features=use_gitea
```

### 基准参考
```
cargo test            ~28s
cargo test (gitea)    ~4s
```

---

*最后更新：2026-03-17*
