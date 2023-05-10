<p align="center">
  <img width="128" alt="mgit logo" src="./imgs/logo-128.png">
</p>

<h1 align="center">MGIT - git 多仓库管理工具</h1>

<p align="center">
  <img width="500" alt="mgit" src="./imgs/mgit-gui.png">
</p>

mgit 是一个用 rust 编写的 git 多仓库管理工具。 他的主要功能有：

- 一键生成当前文件夹下所有仓库的管理配置文件
- 根据配置文件内容，更新指定仓库的 branch，tag 或 commit
- 根据配置文件内容，清理文件夹下的无用仓库
- 提供 cli 工具和 gui 工具

## 命令行工具 (CLI)

```shell
Usage: mgit.exe <COMMAND>

Commands:
  init      Init git repos
  snapshot  Snapshot git repos
  sync      Sync git repos
  fetch     Fetch git repos
  clean     Clean unused git repos
  track     Track remote branch
  ls-files  List files
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### init

```shell
mgit init [OPTIONS] [PATH]
```

初始化指定目录，扫描目录夹下的 git 仓库，并生成配置文件 `.gitrepos`

Options

- **--force** 强制执行并覆盖已有的 `.gitrepos`

### snapshot

```shell
mgit snapshot [OPTIONS] [PATH]
```

快照指定目录，扫描目录夹下的 git 仓库，并将当前 commit 记录生成配置文件

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **--branch** 生成 branch 快照
- **--force** 强制执行并覆盖已有的配置文件
- **--ignore** 忽略不想生成 config 文件的目录，可多次使用

### sync

```shell
mgit sync [OPTIONS] [PATH]
```

通过配置文件，拉取更新仓库。

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **-t, --thread `<NUMBER>`** 设置线程数量，默认是 4
- **--silent** 在 sync 中启用静默播报模式
- **--no-track** 在 sync 后不跟踪远端分支
- **--no-checkout** 在 sync 后不迁出新的远端分支
- **--stash** 在 sync 前暂存本地改动
- **--hard** 在 sync 前忽略所有本地改动
- **--ignore** 忽略不想生成 config 文件的目录，可多次使用
- **--depth** 设置 sync 的深度

### fetch

```shell
mgit fetch [OPTIONS] [PATH]
```

对指定目录执行 `git fetch` 指令

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **-t, --thread `<NUMBER>`** 设置线程数量，默认是 4
- **--silent** 在 sync 中启用静默播报模式
- **--ignore** 忽略不想生成 config 文件的目录，可多次使用
- **--depth** 设置 fetch 深度

### clean

```shell
mgit clean [OPTIONS] [PATH]
```

根据配置文件的仓库路径和指定路径的仓库之间的比对结果，清理不在配置文件中的仓库。

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`

### track

```shell
mgit track [OPTIONS] [PATH]
```

通过配置文件，跟踪远端分支

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **--ignore** 忽略不想生成 config 文件的目录，可多次使用

### ls-files

```shell
mgit ls-files [OPTIONS] [PATH]
```

通过配置文件，浏览本地文件

Options

- **--config `<FILE>`** 指定配置文件，默认找当前目录下的 `.gitrepos`

## 图形界面工具 (GUI)

- 提供勾选界面，方便管理仓库
- 提供菜单选项
- 根据项目保存用户配置
- ...

## 参考

- [git2](https://github.com/rust-lang/git2-rs)
- [clap](https://github.com/clap-rs/clap)
- [git-workspace](https://github.com/orf/git-workspace)
- [git-repo-manager](https://github.com/hakoerber/git-repo-manager)

