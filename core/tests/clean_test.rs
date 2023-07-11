use mgit::ops;
use mgit::ops::CleanOptions;
use std::env;
use std::path::PathBuf;

use crate::common::{exec_cmd, failed_message, TomlBuilder, IMGUI_REPO};

mod common;

/// 测试内容：
///     1、运行命令 mgit sync <path>
///     2、清除不在配置文件 (.gitrepos) 中的仓库
///     3、根目录不是仓库
///
/// 测试目录结构:
///   test_clean1
///     ├─foobar-1 (.git)
///     │  ├──foobar-1-1 (.git)
///     │  │    ├──1.txt
///     │  │    ├──2.txt
///     │  │    ├──foo
///     │  │    │   ├──1.txt
///     │  │    │   └──2.txt
///     │  │    └──bar
///     │  │        ├──1.txt
///     │  │        └──2.txt
///     │  ├──foobar-1-2
///     │  │    └──foobar-1-2-1(.git)
///     │  │        ├──1.txt
///     │  │        ├──2.txt
///     │  │        ├──foo
///     │  │        │   ├──1.txt
///     │  │        │   └──2.txt
///     │  │        └──bar
///     │  │            ├──1.txt
///     │  │            └──2.txt
///     │  ├──1.txt
///     │  ├──2.txt
///     │  ├──foo
///     │  │   ├──1.txt
///     │  │   └──2.txt
///     │  └──bar
///     │       ├──1.txt
///     │       └──2.txt
///     ├─foobar-2 (.git)
///     │  ├──foobar-2-1 (.git)
///     │  │    ├──1.txt
///     │  │    ├──2.txt
///     │  │    ├──foo
///     │  │    │   ├──1.txt
///     │  │    │   └──2.txt
///     │  │    └──bar
///     │  │        ├──1.txt
///     │  │        └──2.txt
///     │  ├──1.txt
///     │  ├──2.txt
///     │  ├──foo
///     │  │   ├──1.txt
///     │  │   └──2.txt
///     │  └──bar
///     │       ├──1.txt
///     │       └──2.txt
///     └─foobar-3 (.git)
///        ├──1.txt
///        ├──2.txt
///        ├──foo
///        │   ├──1.txt
///        │   └──2.txt
///        └──bar
///             ├──1.txt
///             └──2.txt
#[test]
fn cli_clean1() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_clean1");
    let rel_paths = [
        "foobar-1",
        "foobar-1/foobar-1-1",
        "foobar-1/foobar-1-2/foobar-1-2-1",
        "foobar-2",
        "foobar-2/foobar-2-1",
        "foobar-3",
    ];

    create_repos_tree(&path, &rel_paths);

    let config_file = path.join(".gitrepos");
    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(rel_paths[2], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[3], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[4], &IMGUI_REPO, None, None, None)
        .build();

    std::fs::write(&config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    ops::clean_repo(CleanOptions::new(Some(path.clone()), None::<PathBuf>));

    for rel_path in rel_paths {
        let dir = path.join(&rel_path);

        if rel_path == "foobar-1/foobar-1-2/foobar-1-2-1"
            || rel_path == "foobar-2"
            || rel_path == "foobar-2/foobar-2-1"
        {
            assert!(dir.join("1.txt").is_file());
            assert!(dir.join("2.txt").is_file());

            assert!(dir.join(".git").is_dir());

            assert!(dir.join("foo").is_dir());
            assert!(dir.join("foo/1.txt").is_file());
            assert!(dir.join("foo/2.txt").is_file());

            assert!(dir.join("bar").is_dir());
            assert!(dir.join("bar/1.txt").is_file());
            assert!(dir.join("bar/2.txt").is_file());
        } else {
            assert_eq!(false, dir.join("1.txt").is_file());
            assert_eq!(false, dir.join("2.txt").is_file());

            assert_eq!(false, dir.join(".git").is_dir());

            assert_eq!(false, dir.join("foo").is_dir());
            assert_eq!(false, dir.join("foo/1.txt").is_file());
            assert_eq!(false, dir.join("foo/2.txt").is_file());

            assert_eq!(false, dir.join("bar").is_dir());
            assert_eq!(false, dir.join("bar/1.txt").is_file());
            assert_eq!(false, dir.join("bar/2.txt").is_file());
        }
    }

    // clean-up
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit sync <path>
///     2、清除不在配置文件 (.gitrepos) 中的仓库
///     3、根目录是仓库
///
/// 测试目录结构:
///   test_clean2 (.git)
///     ├─... (same as cli_clean1())
#[test]
fn cli_clean2() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_clean2");
    let rel_paths = [
        ".",
        "foobar-1",
        "foobar-1/foobar-1-1",
        "foobar-1/foobar-1-2/foobar-1-2-1",
        "foobar-2",
        "foobar-2/foobar-2-1",
        "foobar-3",
    ];

    create_repos_tree(&path, &rel_paths);

    let config_file = path.join(".gitrepos");
    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(rel_paths[0], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[3], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[4], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[5], &IMGUI_REPO, None, None, None)
        .build();

    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    ops::clean_repo(CleanOptions::new(Some(path.clone()), None::<PathBuf>));

    for rel_path in rel_paths {
        let dir = path.join(&rel_path);

        if rel_path == "."
            || rel_path == "foobar-1/foobar-1-2/foobar-1-2-1"
            || rel_path == "foobar-2"
            || rel_path == "foobar-2/foobar-2-1"
        {
            assert!(dir.join("1.txt").is_file());
            assert!(dir.join("2.txt").is_file());

            assert!(dir.join(".git").is_dir());

            assert!(dir.join("foo").is_dir());
            assert!(dir.join("foo/1.txt").is_file());
            assert!(dir.join("foo/2.txt").is_file());

            assert!(dir.join("bar").is_dir());
            assert!(dir.join("bar/1.txt").is_file());
            assert!(dir.join("bar/2.txt").is_file());
        } else {
            assert_eq!(false, dir.join("1.txt").is_file());
            assert_eq!(false, dir.join("2.txt").is_file());

            assert_eq!(false, dir.join(".git").is_dir());

            assert_eq!(false, dir.join("foo").is_dir());
            assert_eq!(false, dir.join("foo/1.txt").is_file());
            assert_eq!(false, dir.join("foo/2.txt").is_file());

            assert_eq!(false, dir.join("bar").is_dir());
            assert_eq!(false, dir.join("bar/1.txt").is_file());
            assert_eq!(false, dir.join("bar/2.txt").is_file());
        }
    }

    // clean-up
    std::fs::remove_dir_all(&path).unwrap();
}

/// 测试内容：
///     1、运行命令 mgit sync <path> --config <path>
///     2、清除不在配置文件 (.gitrepos) 中的仓库
///     3、根目录不是仓库
///
/// 测试目录结构:
///   test_clean3
///     ├─... (same as cli_clean1())
#[test]
fn cli_clean3() {
    let path = env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("test_clean3");
    let rel_paths = [
        "foobar-1",
        "foobar-1/foobar-1-1",
        "foobar-1/foobar-1-2/foobar-1-2-1",
        "foobar-2",
        "foobar-2/foobar-2-1",
        "foobar-3",
    ];

    create_repos_tree(&path, &rel_paths);

    let config_file = path.join(".gitrepos");
    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(rel_paths[2], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[3], &IMGUI_REPO, None, None, None)
        .join_repo(rel_paths[4], &IMGUI_REPO, None, None, None)
        .build();

    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    let config_path = &path.join(".gitrepos");
    ops::clean_repo(CleanOptions::new(Some(path.clone()), Some(config_path)));

    for rel_path in rel_paths {
        let dir = path.join(&rel_path);

        if rel_path == "foobar-1/foobar-1-2/foobar-1-2-1"
            || rel_path == "foobar-2"
            || rel_path == "foobar-2/foobar-2-1"
        {
            assert!(dir.join("1.txt").is_file());
            assert!(dir.join("2.txt").is_file());

            assert!(dir.join(".git").is_dir());

            assert!(dir.join("foo").is_dir());
            assert!(dir.join("foo/1.txt").is_file());
            assert!(dir.join("foo/2.txt").is_file());

            assert!(dir.join("bar").is_dir());
            assert!(dir.join("bar/1.txt").is_file());
            assert!(dir.join("bar/2.txt").is_file());
        } else {
            assert_eq!(false, dir.join("1.txt").is_file());
            assert_eq!(false, dir.join("2.txt").is_file());

            assert_eq!(false, dir.join(".git").is_dir());

            assert_eq!(false, dir.join("foo").is_dir());
            assert_eq!(false, dir.join("foo/1.txt").is_file());
            assert_eq!(false, dir.join("foo/2.txt").is_file());

            assert_eq!(false, dir.join("bar").is_dir());
            assert_eq!(false, dir.join("bar/1.txt").is_file());
            assert_eq!(false, dir.join("bar/2.txt").is_file());
        }
    }

    // clean-up
    std::fs::remove_dir_all(&path).unwrap();
}

pub fn create_repos_tree(path: &PathBuf, rel_paths: &[&str]) {
    if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
    std::fs::create_dir_all(&path).unwrap();
    let remote = "https://github.com/imgui-rs/imgui-rs.git";

    // create git repos、 some files and some folders
    for rel_path in rel_paths {
        let dir = path.join(rel_path);
        std::fs::create_dir_all(dir.to_path_buf()).unwrap();

        // create local git repositoris
        exec_cmd(&dir, "git", &["init"]).expect(failed_message::GIT_INIT);

        // add remote
        exec_cmd(&dir, "git", &["remote", "add", "origin", remote])
            .expect(failed_message::GIT_ADD_REMOTE);

        // create some files
        std::fs::File::create(dir.join("1.txt")).ok();
        std::fs::File::create(dir.join("2.txt")).ok();

        // create some folders
        std::fs::create_dir_all(dir.join("foo")).ok();
        std::fs::File::create(dir.join("foo/1.txt")).ok();
        std::fs::File::create(dir.join("foo/2.txt")).ok();
        std::fs::create_dir_all(dir.join("bar")).ok();
        std::fs::File::create(dir.join("bar/1.txt")).ok();
        std::fs::File::create(dir.join("bar/2.txt")).ok();
    }
}
