use mgit::error::MgitResult;
use mgit::ops;
use mgit::ops::{InitOptions, SnapshotOptions, SnapshotType};
use mgit::utils::cmd::retry;
use std::path::PathBuf;
use std::time::Duration;

use crate::common::{create_test_dir, exec_cmd, failed_message, TomlBuilder, CSBOOKS_REPO};

mod common;

/// 测试内容�?
///     1、运行命�?mgit init <path>
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、branch
///     3、根目录不是仓库
///     4. 只有同级仓库目录
///
/// 测试目录结构:
///   test_snapshot_init
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_init_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_snapshot_init");
    let path = tmp_dir.path().to_path_buf();

    // Ensure parent directory exists for tmp_dir if it's not the default temp dir
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }

    create_repos_tree1(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), None)).await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit init <path> --force
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、branch
///     3、根目录是仓�?
///     4. 只有同级仓库目录
///
/// 测试目录结构:
///   cli_init_force1 (.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_init_force1() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_init_force1");
    let path = tmp_dir.path().to_path_buf();

    create_repos_tree2(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), Some(true))).await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit init <path> --force
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、branch
///     3、根目录不是仓库
///     4. 具有父子级仓库目�?
///
/// 测试目录结构:
///   cli_init_force2 (.git)
///     ├─foobar-1 (.git)
///     �? ├──foobar-1-1 (.git)
///     �? └──foobar-1-2 (.git)
///     └─foobar-2 (.git)
///        ├──foobar-2-1 (.git)
///        └──foobar-2-2 (.git)
#[tokio::test]
async fn cli_init_force2() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_init_force2");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), None)).await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1/foobar-1-1",
            &CSBOOKS_REPO,
            Some("master"),
            None,
            None,
        )
        .join_repo(
            "foobar-1/foobar-1-2",
            &CSBOOKS_REPO,
            Some("master"),
            None,
            None,
        )
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-2/foobar-2-1",
            &CSBOOKS_REPO,
            Some("master"),
            None,
            None,
        )
        .join_repo(
            "foobar-2/foobar-2-2",
            &CSBOOKS_REPO,
            Some("master"),
            None,
            None,
        )
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit snapshot <path>
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、commit
///     3、根目录不是仓库
#[tokio::test]
async fn cli_snapshot_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_snapshot_simple");
    let path = tmp_dir.path().to_path_buf();

    create_repos_tree1(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        None::<PathBuf>,
        None,
        None,
        None,
    ))
    .await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            "foobar-1",
            &CSBOOKS_REPO,
            None,
            Some("8d90314117b4cb86abb6c4d55130437c6d87a30d"),
            None,
        )
        .join_repo(
            "foobar-2",
            &CSBOOKS_REPO,
            None,
            Some("8d90314117b4cb86abb6c4d55130437c6d87a30d"),
            None,
        )
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit snapshot <path> --branch
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、branch
///     3、根目录不是仓库
#[tokio::test]
async fn cli_snapshot_branch() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_snapshot_branch");
    let path = tmp_dir.path().to_path_buf();

    create_repos_tree1(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        None::<PathBuf>,
        None,
        Some(SnapshotType::Branch),
        None,
    ))
    .await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit snapshot <path> --force --config <path>
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、commit
///     3、根目录是仓�?
#[tokio::test]
async fn cli_snapshot_force() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_snapshot_force");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    let config_file = input_path.clone() + "/.gitrepos";
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        Some(config_file.clone()),
        Some(true),
        None,
        None,
    ))
    .await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(config_file).unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .join_repo(
            "foobar-1",
            &CSBOOKS_REPO,
            None,
            Some("8d90314117b4cb86abb6c4d55130437c6d87a30d"),
            None,
        )
        .join_repo(
            "foobar-1/foobar-1-1",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .join_repo(
            "foobar-1/foobar-1-2",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .join_repo(
            "foobar-2",
            &CSBOOKS_REPO,
            None,
            Some("8d90314117b4cb86abb6c4d55130437c6d87a30d"),
            None,
        )
        .join_repo(
            "foobar-2/foobar-2-1",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .join_repo(
            "foobar-2/foobar-2-2",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

/// 测试内容�?
///     1、运行命�?mgit snapshot <path> --ignore <path> --ignore <path>
///     2、抓�?path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息�?local、remote、commit
///     3、根目录是仓�?
#[tokio::test]
async fn cli_snapshot_ignore() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_snapshot_ignore");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path).await;

    let input_path = path.clone().into_os_string().into_string().unwrap();
    let config_file = input_path.clone() + "/.gitrepos";
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        Some(config_file.clone()),
        Some(true),
        None,
        Some(vec![
            ".".to_string(),
            "foobar-1/foobar-1-2".to_string(),
            "foobar-2".to_string(),
            "foobar-2/foobar-2-2".to_string(),
        ]),
    ))
    .await?;

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(config_file).unwrap();
    let expect_result = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            "foobar-1",
            &CSBOOKS_REPO,
            None,
            Some("8d90314117b4cb86abb6c4d55130437c6d87a30d"),
            None,
        )
        .join_repo(
            "foobar-1/foobar-1-1",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .join_repo(
            "foobar-2/foobar-2-1",
            &CSBOOKS_REPO,
            None,
            Some("1e835f92604ee5d0b37fc32ea7694d57ff19815e"),
            None,
        )
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    Ok(())
}

pub async fn create_repos_tree1(path: &PathBuf) {
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
    std::fs::create_dir_all(path.clone()).unwrap();

    let remote: &str = &CSBOOKS_REPO;
    let commit = "8d90314117b4cb86abb6c4d55130437c6d87a30d";
    let repo_names = ["foobar-1", "foobar-2"];

    for repo_name in repo_names {
        let dir = path.join(repo_name);
        std::fs::create_dir_all(&dir).unwrap();

        // create local git repositoris
        exec_cmd(&dir, "git", &["init", "-b", "master"]).expect(failed_message::GIT_INIT);
        exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
            .expect(failed_message::GIT_ADD_REMOTE);
        retry(10, Duration::from_millis(400), || async {
            exec_cmd(&dir, "git", &["fetch", "origin"]).map_err(|e| {
                mgit::error::MgitError::OpsError {
                    message: e.to_string(),
                }
            })
        })
        .await
        .expect(failed_message::GIT_FETCH);
        exec_cmd(&dir, "git", &["switch", "-c", "master", commit])
            .expect(failed_message::GIT_CHECKOUT);
        exec_cmd(&dir, "git", &["branch", "-u", "origin/master"])
            .expect(failed_message::GIT_BRANCH);
    }
}

pub async fn create_repos_tree2(path: &PathBuf) {
    create_repos_tree1(path).await;

    // create local git repositoris
    let remote: &str = &CSBOOKS_REPO;
    let commit = "1e835f92604ee5d0b37fc32ea7694d57ff19815e";

    exec_cmd(path, "git", &["init", "-b", "master"]).expect(failed_message::GIT_INIT);
    exec_cmd(path, "git", &["remote", "add", "origin", remote])
        .expect(failed_message::GIT_ADD_REMOTE);
    retry(10, Duration::from_millis(400), || async {
        exec_cmd(path, "git", &["fetch", "origin"]).map_err(|e| mgit::error::MgitError::OpsError {
            message: e.to_string(),
        })
    })
    .await
    .expect(failed_message::GIT_FETCH);
    exec_cmd(path, "git", &["switch", "-c", "master", commit]).expect(failed_message::GIT_CHECKOUT);
    exec_cmd(path, "git", &["branch", "-u", "origin/master"]).expect(failed_message::GIT_BRANCH);
}

pub async fn create_repos_tree3(path: &PathBuf) {
    // set root git init
    create_repos_tree1(path).await;

    let remote: &str = &CSBOOKS_REPO;
    let commit = "1e835f92604ee5d0b37fc32ea7694d57ff19815e";

    // get all dir
    for it in std::fs::read_dir(path).unwrap() {
        let dir_entry = match it {
            Ok(dir) => dir,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let entry_path = &dir_entry.path();
        let entry_name = &entry_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // init repo randomly
        for idx in 1..3 {
            let repo_name = format!("{}-{}", entry_name, idx);
            let dir = entry_path.join(&repo_name);

            std::fs::create_dir_all(&dir).unwrap();
            // create local git repositoris
            exec_cmd(&dir, "git", &["init", "-b", "master"]).expect(failed_message::GIT_INIT);
            exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
                .expect(failed_message::GIT_ADD_REMOTE);
            retry(10, Duration::from_millis(400), || async {
                exec_cmd(&dir, "git", &["fetch", "origin"]).map_err(|e| {
                    mgit::error::MgitError::OpsError {
                        message: e.to_string(),
                    }
                })
            })
            .await
            .expect(failed_message::GIT_FETCH);
            exec_cmd(&dir, "git", &["switch", "-c", "master", commit])
                .expect(failed_message::GIT_CHECKOUT);
            exec_cmd(&dir, "git", &["branch", "-u", "origin/master"])
                .expect(failed_message::GIT_BRANCH);
        }
    }

    // set root git init
    exec_cmd(path, "git", &["init", "-b", "master"]).expect(failed_message::GIT_INIT);
    exec_cmd(path, "git", &["remote", "add", "origin", remote])
        .expect(failed_message::GIT_ADD_REMOTE);
    retry(10, Duration::from_millis(400), || async {
        exec_cmd(path, "git", &["fetch", "origin"]).map_err(|e| mgit::error::MgitError::OpsError {
            message: e.to_string(),
        })
    })
    .await
    .expect(failed_message::GIT_FETCH);
    exec_cmd(path, "git", &["switch", "-c", "master", commit]).expect(failed_message::GIT_CHECKOUT);
    exec_cmd(path, "git", &["branch", "-u", "origin/master"]).expect(failed_message::GIT_BRANCH);
}
