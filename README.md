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

- **force** 强制执行并覆盖已有的 `.gitrepos`

### snapshot 

```shell
mgit snapshot [OPTIONS] [PATH]
```

快照指定目录，扫描目录夹下的 git 仓库，并将当前 commit 记录生成配置文件

- **config <FILE>** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **force** 强制执行并覆盖已有的配置文件

Options

- **force** 强制执行并覆盖已有的 `.gitrepos`

### fetch 

```shell
mgit fetch [OPTIONS] [PATH]
```

对指定目录执行 `git fetch` 指令

Options

- **config <FILE>** 指定配置文件，默认找当前目录下的 `.gitrepos`

### sync 

```shell
mgit sync [OPTIONS] [PATH]
```

通过配置文件，拉取更新仓库。

Options

- **config <FILE>** 指定配置文件，默认找当前目录下的 `.gitrepos`
- **force** 清理已存在的修改，并强制设置仓库为配置中的指定版本

### clean 

```shell
mgit clean [OPTIONS] [PATH]
```

根据配置文件的仓库路径和指定路径的仓库之间的比对结果，清理不在配置文件中的仓库。

Options

- **config <FILE>** 指定配置文件，默认找当前目录下的 `.gitrepos`

## 图形界面工具 (GUI)

- 正在开发
  - 提供勾选界面
  - 提供新增 和 删除 界面
  - 提供批量提交界面
  - 提供批量打 tag 界面

## 参考

- [git2](https://github.com/rust-lang/git2-rs)
- [clap](https://github.com/clap-rs/clap)
- [git-workspace](https://github.com/orf/git-workspace)
- [git-repo-manager](https://github.com/hakoerber/git-repo-manager)

