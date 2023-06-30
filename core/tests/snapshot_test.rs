use mgit::ops;
use mgit::ops::{InitOptions, SnapshotOptions, SnapshotType};
use std::env;
use std::path::PathBuf;

use crate::common::{exec_cmd, failed_message, TomlBuilder, CSBOOKS_REPO};

mod common;

/// 测试内容：
///     1、运行命令 mgit init <path>
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、branch
///     3、根目录不是仓库
///     4. 只有同级仓库目录
///
/// 测试目录结构:
///   test_snapshot_init
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[test]
fn cli_init_simple() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_snapshot_init");

    create_repos_tree1(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), None));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::new()
        .default_branch("develop")
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    // std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit init <path> --force
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、branch
///     3、根目录是仓库
///     4. 只有同级仓库目录
///
/// 测试目录结构:
///   cli_init_force1 (.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[test]
fn cli_init_force1() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("cli_init_force1");

    create_repos_tree2(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), Some(true)));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::new()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit init <path> --force
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、branch
///     3、根目录不是仓库
///     4. 具有父子级仓库目录
///
/// 测试目录结构:
///   cli_init_force2 (.git)
///     ├─foobar-1 (.git)
///     │  ├──foobar-1-1 (.git)
///     │  └──foobar-1-2 (.git)
///     └─foobar-2 (.git)
///        ├──foobar-2-1 (.git)
///        └──foobar-2-2 (.git)
#[test]
fn cli_init_force2() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("cli_init_force2");
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::init_repo(InitOptions::new(Some(path.clone()), None));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::new()
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
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit snapshot <path>
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、commit
///     3、根目录不是仓库
#[test]
fn cli_snapshot_simple() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("cli_snapshot_simple");

    create_repos_tree1(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        None::<PathBuf>,
        None,
        None,
        None,
    ));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::new()
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
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit snapshot <path> --branch
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、branch
///     3、根目录不是仓库
#[test]
fn cli_snapshot_branch() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_snapshot_branch");

    create_repos_tree1(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        None::<PathBuf>,
        None,
        Some(SnapshotType::Branch),
        None,
    ));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(input_path + "/.gitrepos").unwrap();
    let expect_result = TomlBuilder::new()
        .default_branch("develop")
        .join_repo("foobar-1", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    assert_eq!(real_result.trim(), expect_result.trim());

    // clean-up
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit snapshot <path> --force --config <path>
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、commit
///     3、根目录是仓库
#[test]
fn cli_snapshot_force() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_snapshot_force");
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    let config_file = input_path.clone() + "/.gitrepos";
    // execute cli init function with path
    ops::snapshot_repo(SnapshotOptions::new(
        Some(path.clone()),
        Some(config_file.clone()),
        Some(true),
        None,
        None,
    ));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(config_file).unwrap();
    let expect_result = TomlBuilder::new()
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
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit snapshot <path> --ignore <path> --ignore <path>
///     2、抓取 path 下的所有仓库信息到配置文件 (.gitrepos)
///        仓库信息为 local、remote、commit
///     3、根目录是仓库
#[test]
fn cli_snapshot_ignore() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_snapshot_ignore");
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path);

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
    ));

    // get content from .gitrepos
    let real_result = std::fs::read_to_string(config_file).unwrap();
    let expect_result = TomlBuilder::new()
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
    std::fs::remove_dir_all(&path).unwrap();
}

pub fn create_repos_tree1(path: &PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path).unwrap();
    }
    std::fs::create_dir_all(path.clone()).unwrap();

    let remote: &str = &CSBOOKS_REPO;
    let commit = "8d90314117b4cb86abb6c4d55130437c6d87a30d";
    let repo_names = ["foobar-1", "foobar-2"];

    for idx in 0..repo_names.len() {
        let dir = path.join(repo_names[idx]);
        std::fs::create_dir_all(dir.to_path_buf()).unwrap();

        // create local git repositoris
        exec_cmd(&dir, "git", &["init"]).expect(failed_message::GIT_INIT);
        exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
            .expect(failed_message::GIT_ADD_REMOTE);
        exec_cmd(&dir, "git", &["fetch", "origin"]).expect(failed_message::GIT_FETCH);
        exec_cmd(&dir, "git", &["switch", "-c", "master", commit])
            .expect(failed_message::GIT_CHECKOUT);
        exec_cmd(&dir, "git", &["branch", "-u", "origin/master"])
            .expect(failed_message::GIT_BRANCH);
    }
}

pub fn create_repos_tree2(path: &PathBuf) {
    create_repos_tree1(path);

    // create local git repositoris
    let remote: &str = &CSBOOKS_REPO;
    let commit = "1e835f92604ee5d0b37fc32ea7694d57ff19815e";

    exec_cmd(&path, "git", &["init"]).expect(failed_message::GIT_INIT);
    exec_cmd(&path, "git", &["remote", "add", "origin", remote])
        .expect(failed_message::GIT_ADD_REMOTE);
    exec_cmd(&path, "git", &["fetch", "origin"]).expect(failed_message::GIT_FETCH);
    exec_cmd(&path, "git", &["switch", "-c", "master", commit])
        .expect(failed_message::GIT_CHECKOUT);
    exec_cmd(&path, "git", &["branch", "-u", "origin/master"]).expect(failed_message::GIT_BRANCH);
}

pub fn create_repos_tree3(path: &PathBuf) {
    // set root git init
    create_repos_tree1(path);

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

            std::fs::create_dir_all(dir.to_path_buf()).unwrap();
            // create local git repositoris
            exec_cmd(&dir, "git", &["init"]).expect(failed_message::GIT_INIT);
            exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
                .expect(failed_message::GIT_ADD_REMOTE);
            exec_cmd(&dir, "git", &["fetch", "origin"]).expect(failed_message::GIT_FETCH);
            exec_cmd(&dir, "git", &["switch", "-c", "master", commit])
                .expect(failed_message::GIT_CHECKOUT);
            exec_cmd(&dir, "git", &["branch", "-u", "origin/master"])
                .expect(failed_message::GIT_BRANCH);
        }
    }

    // set root git init
    exec_cmd(&path, "git", &["init"]).expect(failed_message::GIT_INIT);
    exec_cmd(&path, "git", &["remote", "add", "origin", remote])
        .expect(failed_message::GIT_ADD_REMOTE);
    exec_cmd(&path, "git", &["fetch", "origin"]).expect(failed_message::GIT_FETCH);
    exec_cmd(&path, "git", &["switch", "-c", "master", commit])
        .expect(failed_message::GIT_CHECKOUT);
    exec_cmd(&path, "git", &["branch", "-u", "origin/master"]).expect(failed_message::GIT_BRANCH);
}
