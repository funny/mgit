use mgit::error::MgitResult;
use mgit::ops;
use mgit::ops::SyncOptions;
use std::{collections::HashSet, path::PathBuf};

use crate::common::{
    create_test_dir, exec_cmd, failed_message, TestProgress, TomlBuilder, CSBOOKS_REPO,
    DEFAULT_BRANCH, SBERT_REPO,
};

mod common;

/// 测试内容：
///     1、运行命令 mgit sync <path> --no-checkout
///     2、批量同步配置文件 (.gitrepos)所有仓库，模拟 git reset --soft 到远端 commit/tag/branch
///     3、local commit 会还原成 local changes
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_simple(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_simple");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            Some(true),
            None,
        ),
        TestProgress,
    )
    .await?;
    // ignore "foobar" folder
    let ignore_file = path.join(".gitignore");
    let ingore_content = "foobar";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // create new local commit
    std::fs::File::create(path.join("1.txt")).ok();
    let _ = exec_cmd(&path, "git", &["add", ".", "-f"]);
    check_git_author_identity(&path);
    let _ = exec_cmd(&path, "git", &["commit", "-am", "foobar"]);

    // if commit succ
    assert!(exec_cmd(&path, "git", &["status"]).is_ok());

    // sync --hard will delete .gitrepos in the front
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    // excute sync
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    // compaire changes after sync
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains(".gitrepos"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --no-checkout
///     2、模拟远端 commit/tag/branch 失效时的情况
///     3、commit 会还原成 local changes
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_tracking_invalid(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_simple_tracking_invalid() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_simple_tracking_invalid");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // ignore "foobar" folder
    let ignore_file = path.join(".gitignore");
    let ingore_content = "foobar";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // create new local commit
    std::fs::File::create(path.join("1.txt")).ok();

    // compaire changes now
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    // set invalid branch
    let toml_string = toml_string.replace("master", "invalid-branch");
    // sync --hard will delete .gitrepos in the front
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    // excute sync
    let result = ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
        ),
        TestProgress,
    )
    .await;
    assert!(result.is_err());

    // compaire changes after sync
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --stash
///     2、测试 sync 时，先 stash local changes，再 reset 到远端 commit/tag/branch
///     3、根目录是仓库
///
/// 测试目录结构:
///   test_sync_stash(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_stash() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_stash");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // ignore "foobar" folder
    let ignore_file = path.join(".gitignore");
    let ingore_content = "foobar";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // create new local commit
    std::fs::File::create(path.join("1.txt")).ok();

    // compaire changes now
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains("1.txt"))
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    // sync --hard will delete .gitrepos in the front
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // excute sync
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    // compaire changes after sync
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(!output.contains(".gitignore"));
        assert!(!output.contains(".gitrepos"));
        assert!(!output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    // check stash
    if let Ok(output) = exec_cmd(&path, "git", &["stash", "list"]) {
        assert_eq!(1, output.lines().count());
    } else {
        panic!("{}", failed_message::GIT_STASH_LIST);
    }

    // pop stash and check file
    if let Ok(output) = exec_cmd(&path, "git", &["stash", "pop"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains(".gitrepos"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STASH_POP);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --stash
///     2、测试远端 commit/tag/branch 失时导致 sync 失败，将 stash 的内容（若有的话）pop 出来
///     3、根目录是仓库
///
/// 测试目录结构:
///   test_sync_stash(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_stash_tracking_invalid() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_stash_tracking_invalid");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // ignore "foobar" folder
    let ignore_file = path.join(".gitignore");
    let ingore_content = "foobar";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // create new local commit
    std::fs::File::create(path.join("1.txt")).ok();

    // compaire changes now
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    // set invalid branch
    let toml_string = toml_string.replace("master", "invalid-branch");
    // sync --hard will delete .gitrepos in the front
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    // excute sync --stash

    let result = ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
            None,
        ),
        TestProgress,
    )
    .await;
    assert!(result.is_err());

    // compaire changes after sync
    // nothing will be stash
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(output.contains(".gitrepos"));
        assert!(output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --hard
///     2、测试 sync后, 丢弃所有 changes
///     3、根目录是仓库
///
/// 测试目录结构:
///   test_sync_hard(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_hard() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_hard");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // ignore "foobar" folder
    let ignore_file = path.join(".gitignore");
    let ingore_content = "foobar";
    std::fs::write(&ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // create new local commit
    std::fs::File::create(path.join("1.txt")).ok();
    let _ = exec_cmd(&path, "git", &["add", ".", "-f"]);
    check_git_author_identity(&path);
    let _ = exec_cmd(&path, "git", &["commit", "-am", "foobar"]);

    // if commit succ
    assert!(exec_cmd(&path, "git", &["status"]).is_ok());

    // sync --hard will delete .gitrepos in the front
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    // excute sync --hard
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    // ignore "foobar" folder
    std::fs::write(&ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);
    // compaire changes after sync
    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(output.contains(".gitignore"));
        assert!(!output.contains(".gitrepos"));
        assert!(!output.contains("1.txt"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --config
///     2、path 指向的目录不存在时，自动创建目录
///     3、sync 后，changes 数量为 0
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_simple_invalid_path
///     ├─.gitrepos
///     └─foobar-1
///         ├─foobar-1-1 (.git)
///         └─1.txt
#[tokio::test]
async fn cli_sync_simple_invalid_path() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_simple_invalid_path");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(&path).unwrap();

    let input_path = path.join("foobar-1");

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar-1-1", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    assert!(config_file.is_file());
    assert!(!input_path.is_dir());

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path.clone()),
            Some(config_file.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    assert!(input_path.is_dir());

    // ignore "foobar" folder
    let ignore_file = input_path.join(".gitignore");
    let ingore_content = "foobar-1-1";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // for foobar-1, local changes only contain ".gitignore"
    let local_changes1 = get_local_changes(&input_path);

    assert_eq!(1, local_changes1.len());
    assert!(local_changes1.contains(".gitignore"));

    // for foobar-1/foobar-1-1, local changes is empty
    let local_changes2 = get_local_changes(&input_path.join("foobar-1-1"));
    assert!(local_changes2.is_empty());

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --config --stash
///     2、path 指向的目录不存在时，自动创建目录
///     3、sync 后，changes 数量为 0
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_stash_invalid_path
///     ├─.gitrepos
///     └─foobar-1
///         ├─foobar-1-1 (.git)
///         └─1.txt
#[tokio::test]
async fn cli_sync_stash_invalid_path() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_stash_invalid_path");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(&path).unwrap();

    let input_path = path.join("foobar-1");

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar-1-1", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    assert!(config_file.is_file());
    assert!(!input_path.is_dir());

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path.clone()),
            Some(config_file.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    assert!(input_path.is_dir());

    // ignore "foobar" folder
    let ignore_file = input_path.join(".gitignore");
    let ingore_content = "foobar-1-1";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // for foobar-1, local changes only contain ".gitignore"
    let local_changes1 = get_local_changes(&input_path);
    assert_eq!(1, local_changes1.len());
    assert!(local_changes1.contains(".gitignore"));

    // for foobar-1/foobar-1-1, local changes is empty
    let local_changes2 = get_local_changes(&input_path.join("foobar-1-1"));
    assert!(local_changes2.is_empty());

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --config --hard
///     2、path 指向的目录不存在时，自动创建目录
///     3、sync 后，changes 数量为 0
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_hard_invalid_path
///     ├─.gitrepos
///     └─foobar-1
///         ├─foobar-1-1 (.git)
///         └─1.txt
#[tokio::test]
async fn cli_sync_hard_invalid_path() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_hard_invalid_path");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(&path).unwrap();

    let input_path = path.join("foobar-1");

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar-1-1", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    assert!(config_file.is_file());
    assert!(!input_path.is_dir());

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path.clone()),
            Some(config_file.clone()),
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    assert!(input_path.is_dir());

    // ignore "foobar" folder
    let ignore_file = input_path.join(".gitignore");
    let ingore_content = "foobar-1-1";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // for foobar-1, local changes only contain ".gitignore"
    let local_changes1 = get_local_changes(&input_path);
    assert_eq!(1, local_changes1.len());
    assert!(local_changes1.contains(".gitignore"));

    // for foobar-1/foobar-1-1, local changes is empty
    let local_changes2 = get_local_changes(&input_path.join("foobar-1-1"));
    assert!(local_changes2.is_empty());

    Ok(())
}

/// get local changes
fn get_local_changes(input_path: &PathBuf) -> HashSet<String> {
    let mut changed_files: HashSet<String> = HashSet::new();

    // get untracked files (uncommit)
    let args = ["ls-files", ".", "--exclude-standard", "--others"];
    if let Ok(output) = exec_cmd(input_path, "git", &args) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    // get tracked and changed files (uncommit)
    let args = ["diff", "--name-only"];
    if let Ok(output) = exec_cmd(input_path, "git", &args) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    // get cached(staged) files (uncommit)
    let args = ["diff", "--cached", "--name-only"];
    if let Ok(output) = exec_cmd(input_path, "git", &args) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    changed_files
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --config
///     2、path 指向的目录存在, 但仓库不存在
///     3、sync 后，changes 数量为 0
///     4、根目录是仓库
///
/// 测试目录结构:
///   cli_sync_simple_repo_invalid
///     ├─.gitrepos
///     └─foobar-1
///         ├─foobar-1-1 (.git)
///         └─1.txt
#[tokio::test]
async fn cli_sync_simple_repo_invalid() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_simple_repo_invalid");
    let path = tmp_dir.path().to_path_buf();
    std::fs::create_dir_all(&path).unwrap();

    let input_path = path.join("foobar-1");

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar-1-1", &CSBOOKS_REPO, Some("master"), None, None)
        .build();

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    assert!(config_file.is_file());
    assert!(!input_path.is_dir());

    // create root path: "foobar-1"
    std::fs::create_dir_all(&input_path).unwrap();
    assert!(input_path.is_dir());

    // create path: "foobar-1/foobar-1-1"
    std::fs::create_dir_all(input_path.join("foobar-1-1")).unwrap();
    assert!(input_path.is_dir());

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path.clone()),
            Some(config_file.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    assert!(input_path.is_dir());

    // ignore "foobar" folder
    let ignore_file = input_path.join(".gitignore");
    let ingore_content = "foobar-1-1";
    std::fs::write(ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // for foobar-1, local changes only contain ".gitignore"
    let local_changes1 = get_local_changes(&input_path);

    assert_eq!(1, local_changes1.len());
    assert!(local_changes1.contains(".gitignore"));

    // for foobar-1/foobar-1-1, local changes is empty
    let local_changes2 = get_local_changes(&input_path.join("foobar-1-1"));
    assert!(local_changes2.is_empty());

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit sync <path>
///     2、--no-checkout == false 时,负责创建切换 branch, 不负责 track。
///         - sync commit, 创建或切换 commits/xxxx 分支
///         - sync tag, 创建或切换 tags/x.x.x 分支
///         - sync branch, 创建或切换至同名 branch
///     3、--no-check == false 时, 负责 track
///     4、根目录是仓库,且根目录为空目录
///
/// 测试目录结构:
///   test_sync_checkout_invalid_path(.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_sync_checkout_invalid_path() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_checkout_invalid_path");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1",
            &SBERT_REPO,
            None,
            Some("dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d"),
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, None, None, Some("v0.3.0"))
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    let cur_branch_args = ["branch", "--show-current"];
    let tracking_args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];
    let root_path = &path;
    let foobar_1_path = &path.join("foobar-1");
    let foobar_2_path = &path.join("foobar-2");
    let invald_name = "invalid".to_string();

    // check initial state
    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "master");
    let tracking_branch = exec_cmd(root_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/master");

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "commits/dc1d3db");
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "tags/v0.3.0");
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit sync <path> --no-checkout --no-check
///         - mgit sync <path> --no-track
///         - mgit sync <path>
///         - mgit sync <path> --hard
///     2、--no-checkout == false 时,负责创建切换 branch, 不负责 track。
///         - sync commit, 创建或切换 commits/xxxx 分支
///         - sync tag, 创建或切换 tags/x.x.x 分支
///         - sync branch, 创建或切换至同名 branch
///     3、--no-check == false 时, 负责 track
///     4、根目录是仓库
///
/// 测试目录结构:
///   test_sync_checkout_simple(.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_sync_checkout_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_checkout_simple");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &SBERT_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    let cur_branch_args = ["branch", "--show-current"];
    let tracking_args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];
    let root_path = &path;
    let foobar_1_path = &path.join("foobar-1");
    let foobar_2_path = &path.join("foobar-2");
    let invald_name = "invalid".to_string();

    // check initial state
    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: master untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: master untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1",
            &SBERT_REPO,
            None,
            Some("dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d"),
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, None, None, Some("v0.3.0"))
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // test checkout function
    // root path, checkout a new branch
    // root: foobar untracked
    exec_cmd(
        root_path,
        "git",
        &["checkout", "-B", "foobar", "origin/master", "--no-track"],
    )
    .expect(failed_message::GIT_CHECKOUT);

    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "foobar");
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // sync repositories, with checkout
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
        ),
        TestProgress,
    )
    .await?;

    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "master");
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "commits/dc1d3db");
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "tags/v0.3.0");
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    // sync repositories, with checkout and track
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // test checkout and track function
    // root: master → origin/master
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "master");
    let tracking_branch = exec_cmd(root_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/master");

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "commits/dc1d3db");
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "tags/v0.3.0");
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit sync <path> --no-checkout --no-check
///         - mgit sync <path> --no-track
///         - mgit sync <path>
///         - mgit sync <path> --hard
///     2、--no-checkout == false 时,负责创建切换 branch, 不负责 track
///         - sync commit, 创建或切换 commits/xxxx 分支
///         - sync tag, 创建或切换 tags/x.x.x 分支
///         - sync branch, 创建或切换至同名 branch
///     3、--no-check == false 时, 负责 track
///     4、--hard == false 时，有冲突会 checkout 失败
///        --hard == true 时，放弃 chenges ，强制checkout
///     5、根目录是仓库
///
/// 测试目录结构:
///   test_sync_checkout_with_conflict(.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_sync_checkout_with_conflict() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_checkout_with_conflict");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &SBERT_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    let cur_branch_args = ["branch", "--show-current"];
    let tracking_args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];
    let root_path = &path;
    let foobar_1_path = &path.join("foobar-1");
    let foobar_2_path = &path.join("foobar-2");
    let invald_name = "invalid".to_string();

    // check initial state
    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: master untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: master untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    // ignore and .gitrepos, for confliction test
    let ignore_file = path.join(".gitignore");
    let ingore_content = format!("{}\n{}", "/foobar-1", "/foobar-2");
    std::fs::write(&ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // create local commit
    let _ = exec_cmd(&path, "git", &["add", ".", "-f"]);
    check_git_author_identity(&path);
    let _ = exec_cmd(&path, "git", &["commit", "-am", "foobar"]);

    if let Ok(output) = exec_cmd(&path, "git", &["status"]) {
        assert!(!output.contains(".gitignore"));
        assert!(!output.contains(".gitrepos"));
    } else {
        panic!("{}", failed_message::GIT_STATUS);
    }

    // root path, set checkout a new branch
    // root: foobar untracked
    exec_cmd(
        root_path,
        "git",
        &["checkout", "-B", "foobar", "origin/master", "--no-track"],
    )
    .expect(failed_message::GIT_CHECKOUT);

    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "foobar");
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // for confliction test
    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1",
            &SBERT_REPO,
            None,
            Some("dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d"),
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, None, None, Some("v0.3.0"))
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    std::fs::write(&ignore_file, ingore_content.trim()).expect(failed_message::WRITE_FILE);

    // sync repositories, with checkout
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
        ),
        TestProgress,
    )
    .await?;

    // root: foobar untracked,  checkout failed
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "master");
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "commits/dc1d3db");
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "tags/v0.3.0");
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    // sync repositories, with checkout, track and hard
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // root: master → origin/master
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "master");
    let tracking_branch = exec_cmd(root_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/master");

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "commits/dc1d3db");
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), "tags/v0.3.0");
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit sync <path> --stash
///     2、默认使用 checkout 时，不应存在 local changes
///
/// 测试目录结构:
///   test_sync_checkout_simple2(.git)
#[tokio::test]
async fn cli_sync_checkout_simple2() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_checkout_simple2");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    exec_cmd(&path, "git", &["reset", "--hard", "v0.3.0"]).expect(failed_message::GIT_RESET);

    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // test checkout function
    // sync repositories, with checkout
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // for foobar-1, local changes only contain ".gitignore"
    let local_changes1 = get_local_changes(&path);
    assert_eq!(0, local_changes1.len());

    let output = exec_cmd(&path, "git", &["stash", "list"]).expect(failed_message::GIT_STASH_LIST);
    assert!(output.contains("stash@{0}"));
    assert!(!output.contains("stash@{1}"));

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit sync <path> --igonre <path> --igonre <path>
///     2、--ignore 忽略对应的仓库
///     3、根目录是仓库
///
/// 测试目录结构:
///   test_sync_ignore_simple(.git)
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_sync_ignore_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_ignore_simple");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-1", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1/foobar-1-1",
            &SBERT_REPO,
            Some("master"),
            None,
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-2/foobar-2-1",
            &SBERT_REPO,
            Some("master"),
            None,
            None,
        )
        .join_repo(
            "foobar-2/foobar-2-2",
            &SBERT_REPO,
            Some("master"),
            None,
            None,
        )
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            Some(
                [".", "foobar-1", "foobar-2/foobar-2-1"]
                    .map(|x| x.to_string())
                    .to_vec(),
            ),
            None,
            None,
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    let root_path = path.join(".git");
    let foobar_1_path = path.join("foobar-1/.git");
    let foobar_1_1_path = path.join("foobar-1/foobar-1-1/.git");
    let foobar_2_path = path.join("foobar-2/.git");
    let foobar_2_1_path = path.join("foobar-2/foobar-2-1/.git");
    let foobar_2_2_path = path.join("foobar-2/foobar-2-2/.git");

    assert!(!root_path.is_dir());
    assert!(!foobar_1_path.is_dir());
    assert!(foobar_1_1_path.is_dir());
    assert!(foobar_2_path.is_dir());
    assert!(!foobar_2_1_path.is_dir());
    assert!(foobar_2_2_path.is_dir());

    Ok(())
}

pub fn check_git_author_identity(path: &PathBuf) {
    if exec_cmd(path, "git", &["config", "--global", "user.email"]).is_err() {
        exec_cmd(
            path,
            "git",
            &["config", "--global", "user.email", "foobar@xmfunny.com"],
        )
        .expect(failed_message::GIT_CONFIG);
        exec_cmd(path, "git", &["config", "--global", "user.name", "foobar"])
            .expect(failed_message::GIT_CONFIG);
    }
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --depth 1
///     2、得到深度为 1 的 commit
///
/// 测试目录结构:
///   test_sync_with_depth(.git)
///     ├─foobar (.git)
///     └─1.txt
#[tokio::test]
async fn cli_sync_with_depth() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_sync_with_depth");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .join_repo(
            ".",
            &SBERT_REPO,
            None,
            Some("dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d"),
            None,
        )
        .join_repo("foobar-1", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar-2", &SBERT_REPO, None, None, Some("v0.3.0"))
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            Some(1),
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            Some(1),
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    // get root repo commit count
    if let Ok(output) = exec_cmd(&path, "git", &["rev-list", "--all", "--count"]) {
        assert_eq!(output.trim(), 1.to_string());
    } else {
        panic!("{}", failed_message::GIT_REV_LIST);
    }

    // get foobar-1 repo commit count
    let foobar_path = path.join("foobar-1");
    if let Ok(output) = exec_cmd(foobar_path, "git", &["rev-list", "--all", "--count"]) {
        assert_eq!(output.trim(), 1.to_string());
    } else {
        panic!("{}", failed_message::GIT_REV_LIST);
    }

    // get foobar-2 repo commit count
    let foobar_path = path.join("foobar-2");
    if let Ok(output) = exec_cmd(foobar_path, "git", &["rev-list", "--all", "--count"]) {
        assert_eq!(output.trim(), 1.to_string());
    } else {
        panic!("{}", failed_message::GIT_REV_LIST);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path>
///     2、更新配置文件中新的 url
#[tokio::test]
async fn cli_sync_new_remote_url() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_sync_new_remote_url");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .join_repo(
            ".",
            &CSBOOKS_REPO,
            None,
            Some("fc8ba56c64b7b7e4dd2d171fd95ca620aa36d695"),
            None,
        )
        .join_repo("foobar", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        ),
        TestProgress,
    )
    .await?;

    let repo_paths = ["", "foobar"];
    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        let args = ["config", "--get", "remote.origin.url"];
        let output = exec_cmd(&dir, "git", &args).unwrap_or(String::from("invalid url"));
        assert_eq!(output.trim(), &CSBOOKS_REPO as &str);
    }

    let toml_string = TomlBuilder::default()
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo("foobar", &SBERT_REPO, Some("master"), None, None)
        .build();
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
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
        assert_eq!(output.trim(), &SBERT_REPO as &str);
    }

    Ok(())
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --no-checkout
///     2、检查配置的稀疏检出是否和预期匹配
///     3、根目录是仓库
///
/// 测试目录结构:
///   cli_sync_with_sparse_checkout(.git)
///     ├─Doc
///     ├─img
///     └─README.md
#[tokio::test]
async fn cli_sync_with_sparse_checkout() -> MgitResult<()> {
    let tmp_dir = create_test_dir("cli_sync_with_sparse_checkout");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let mut toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &CSBOOKS_REPO, Some("master"), None, None)
        .build();
    let sparse = r#"sparse = ["Doc", "/*.md"]"#;
    toml_string.push_str(sparse);

    let config_file = path.join(".gitrepos");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize the repositories tree
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            Some(true),
            None,
        ),
        TestProgress,
    )
    .await?;

    // compaire sparse-checkout list
    if let Ok(output) = exec_cmd(&path, "git", &["sparse-checkout", "list"]) {
        assert_eq!(output.contains("Doc"), true);
        assert_eq!(output.contains("img"), false);
        assert_eq!(output.contains("/*.md"), true);

        assert_eq!(path.join("Doc").exists(), true);
        assert_eq!(path.join("img").exists(), false);
        assert_eq!(path.join("README.md").exists(), true);
    } else {
        panic!("{}", failed_message::GIT_SPARSE_CHECKOUT);
    }

    toml_string = toml_string.replace(sparse, "");
    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);
    // excute sync
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    // compaire sparse-checkout list
    let res = exec_cmd(&path, "git", &["sparse-checkout", "list"]);
    assert!(res.is_err());
    assert_eq!(path.join("Doc").exists(), true);
    assert_eq!(path.join("img").exists(), true);
    assert_eq!(path.join("README.md").exists(), true);

    Ok(())
}
