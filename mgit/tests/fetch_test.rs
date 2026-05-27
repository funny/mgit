use mgit::error::MgitResult;
use mgit::ops;
use mgit::ops::{FetchOptions, InitOptions};
use std::path::PathBuf;

use crate::common::{
    create_test_dir, csbooks_repo, exec_cmd, failed_message, mgit_repo, TestProgress, TomlBuilder,
};

mod common;

/// 测试内容：
///     1、运行命令:
///         - mgit fetch <path>
///     5、根目录是仓库
///
/// 测试目录结构:
///   test_fetch_simple(.git)
///     ├─.gitrepos
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_fetch_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_fetch_simple");
    let path = tmp_dir.path().to_path_buf();

    let remote = csbooks_repo();

    let repo_paths = ["", "foobar-1", "foobar-2"];

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        std::fs::create_dir_all(&dir).unwrap();

        // create local git repositoris
        exec_cmd(&dir, "git", &["init"]).expect(failed_message::GIT_INIT);

        // add remote
        exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
            .expect(failed_message::GIT_ADD_REMOTE);

        assert!(!dir.join(".git/FETCH_HEAD").is_file());
    }

    // init command
    ops::init_repo(InitOptions::new(Some(path.clone()), None)).await?;
    // fetch command
    ops::fetch_repos(
        FetchOptions::new(
            Some(path.clone()),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        assert!(dir.join(".git/FETCH_HEAD").is_file());
    }
    Ok(())
}

/// 测试内容：
///     1、运行命令: mgit fetch <path>
///     2、仓库的 remote url 变更配置文件中新的 url
#[tokio::test]
async fn cli_fetch_new_remote_url() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_fetch_new_remote_url");
    let path = tmp_dir.path().to_path_buf();

    let remote = csbooks_repo();

    let repo_paths = ["", "foobar-1"];

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        std::fs::create_dir_all(&dir).unwrap();

        // create local git repositoris
        exec_cmd(&dir, "git", &["init"]).expect(failed_message::GIT_INIT);

        // add remote
        exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
            .expect(failed_message::GIT_ADD_REMOTE);

        assert!(!dir.join(".git/FETCH_HEAD").is_file());
    }

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", mgit_repo(), None, None, None)
        .join_repo("foobar-1", mgit_repo(), None, None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // fetch command
    ops::fetch_repos(
        FetchOptions::new(
            Some(path.clone()),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        let args = ["config", "--get", "remote.origin.url"];
        let output = exec_cmd(&dir, "git", &args).unwrap_or(String::from("invalid url"));
        assert_eq!(output.trim(), mgit_repo());
    }
    Ok(())
}
