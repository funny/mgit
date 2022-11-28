use crate::common::{execute_cargo_cmd, execute_cmd};
use std::env;

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
#[test]
fn cli_fetch_simple() {
    let path = env::current_dir()
        .unwrap()
        .join("target/tmp/test_fetch_simple");
    let input_path = path.to_str().unwrap();

    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();

    let remote = "https://gitee.com/ForthEspada/CS-Books.git";

    let repo_paths = ["", "foobar-1", "foobar-2"];

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        std::fs::create_dir_all(dir.to_path_buf()).unwrap();

        // create local git repositoris
        let _ = execute_cmd(&dir, "git", &["init"]);

        // add remote
        let _ = execute_cmd(&dir, "git", &["remote", "add", "origin", remote]);

        assert!(!dir.join(".git/FETCH_HEAD").is_file());
    }

    // init command
    execute_cargo_cmd("mgit", &["init", &input_path]);
    // fetch command
    execute_cargo_cmd("mgit", &["fetch", &input_path]);

    for repo_path in repo_paths {
        let dir = path.join(repo_path);
        assert!(dir.join(".git/FETCH_HEAD").is_file());
    }
    // clean-up;
    std::fs::remove_dir_all(&path).unwrap();
}
