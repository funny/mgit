mgit 是一个用 rust 编写的，git 多仓库管理工具

- 提供 cli 和 gui 界面
- 提供批量 fetch，update 功能
- 提供一键清理仓库功能
- 通过解析 toml 文件获取仓库本地与远程地址
- 通过 .repo/ 存储用户自定义配置（包括分支名，版本 tag 等）

### cli

- **mgit init ./foo/bar** 初始化项目，检索指定文件夹下的 git 仓库情况，构建 .gitrepos 文件
- **mgit sync** 根据 .gitrepos 和 ./repos 下的情况更新仓库
- **mgit clean --force** 根据 .gitrepos 清理不在其下的 git 仓库，如果使用 —force 则不询问直接删除
- **mgit fetch** 只执行 fetch 更新

### gui

- 提供勾选界面
- 提供新增 和 删除 界面
- 提供批量提交界面
- 提供批量打 tag 界面

### 参考

- [https://docs.rs/git2/latest/git2/](https://docs.rs/git2/latest/git2/)
- [https://docs.rs/clap/latest/clap/](https://docs.rs/clap/latest/clap/)
