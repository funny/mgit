//! git init test
//! use following command:
//!     cargo test --test cli_init -- cli_init_no_args
//!     cargo test --test cli_init -- cli_init_force
//!     cargo test --test cli_init -- cli_init_with_path_only
//!     cargo test --test cli_init -- cli_init_with_path_force1
//!     cargo test --test cli_init -- cli_init_with_path_force2
//!     cargo test --test cli_init -- cli_init_with_path_force3
//! or:
//!     cargo test --test cli_init -- cli_init_no_args --show-output
//!     cargo test --test cli_init -- cli_init_force --show-output
//!     cargo test --test cli_init -- cli_init_with_path_only --show-output
//!     cargo test --test cli_init -- cli_init_with_path_force1 --show-output
//!     cargo test --test cli_init -- cli_init_with_path_force2 --show-output
//!     cargo test --test cli_init -- cli_init_with_path_force3 --show-output
//!
use assert_cmd::prelude::*;
use owo_colors::OwoColorize;
use rand::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_init_no_args() {
    excute_cargo_cmd("mgit", &["init"]);
}

#[test]
fn cli_init_force() {
    excute_cargo_cmd("mgit", &["init", "--force"]);
}

#[test]
fn cli_init_with_path_only() {
    let path = env::current_dir().unwrap().join("target\\tmp\\test_repos");

    create_repos_tree1(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // excute cli init function with path
    excute_cargo_cmd("mgit", &["init", &input_path]);

    // print content from .gitrepos
    println!(
        "content from .gitrepos:\n{}",
        std::fs::read_to_string(input_path + "\\.gitrepos")
            .unwrap()
            .blue()
    );
}

#[test]
fn cli_init_with_path_force1() {
    let path = env::current_dir().unwrap().join("target\\tmp\\test_repos");

    create_repos_tree1(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // excute cli init function with path
    excute_cargo_cmd("mgit", &["init", &input_path, "--force"]);

    // print content from .gitrepos
    println!(
        "content from .gitrepos:\n{}",
        std::fs::read_to_string(input_path + "\\.gitrepos")
            .unwrap()
            .blue()
    );
}

#[test]
fn cli_init_with_path_force2() {
    let path = env::current_dir().unwrap().join("target\\tmp\\test_repos");

    create_repos_tree2(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // excute cli init function with path
    excute_cargo_cmd("mgit", &["init", &input_path, "--force"]);

    // print content from .gitrepos
    println!(
        "content from .gitrepos:\n{}",
        std::fs::read_to_string(input_path + "\\.gitrepos")
            .unwrap()
            .blue()
    );
}

#[test]
fn cli_init_with_path_force3() {
    let path = env::current_dir().unwrap().join("target\\tmp\\test_repos");
    std::fs::create_dir_all(path.clone()).unwrap();

    create_repos_tree3(&path);

    let input_path = path.clone().into_os_string().into_string().unwrap();
    // excute cli init function with path
    excute_cargo_cmd("mgit", &["init", &input_path, "--force"]);

    // print content from .gitrepos
    println!(
        "content from .gitrepos:\n{}",
        std::fs::read_to_string(input_path + "\\.gitrepos")
            .unwrap()
            .blue()
    );
}

pub fn excute_cmd(path: &PathBuf, cmd: &str, args: &[&str]) {
    std::process::Command::new(cmd)
        .current_dir(path.to_path_buf())
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn excute_cargo_cmd(cmd: &str, args: &[&str]) {
    Command::cargo_bin(cmd)
        .unwrap()
        .args(args)
        .assert()
        .success();
}

/// repos tree:
///     test_repos
///     ├─imgui-rs (.git)
///     ├─indicatif.git (.git)
///     ├─git-workspace.git (.git)
///     └─git-repo-manager (.git)
pub fn create_repos_tree1(path: &PathBuf) {
    std::fs::remove_dir_all(path).unwrap();
    std::fs::create_dir_all(path.clone()).unwrap();

    let remotes = vec![
        "https://github.com/imgui-rs/imgui-rs.git",
        "https://github.com/console-rs/indicatif.git",
        "https://github.com/orf/git-workspace.git",
        "https://github.com/hakoerber/git-repo-manager.git",
    ];

    for idx in 0..remotes.len() {
        let pos = remotes[idx].rfind("/").unwrap();
        let (_, repo_name) = remotes[idx].split_at(pos + 1);
        let dir = path.join(&repo_name);
        std::fs::create_dir_all(dir.to_path_buf()).unwrap();

        // create local git repositoris
        excute_cmd(&dir, "git", &["init"]);

        // add remote
        excute_cmd(&dir, "git", &["remote", "add", "origin", remotes[idx]]);
    }
}

/// repos tree: (.git)
///     test_repos (.git)
///     ├─imgui-rs (.git)
///     ├─indicatif.git (.git)
///     ├─git-workspace.git (.git)
///     └─git-repo-manager (.git)
pub fn create_repos_tree2(path: &PathBuf) {
    create_repos_tree1(path);

    // set root git init
    excute_cmd(path, "git", &["init"]);
    let root_remote = "https://github.com/rust-lang/git2-rs.git";
    excute_cmd(path, "git", &["remote", "add", "origin", root_remote]);
}

/// repos tree:
///     test_repos (.git)
///     ├─imgui-rs (.git)
///     │  ├──random_repo1 (.git)
///     │  └──random_repo2 (.git)
///     ├─indicatif.git (.git)
///     │  └─random_repo (.git)
///     ├─git-workspace.git (.git)
///     │  └─random_repo (.git)
///     └─git-repo-manager (.git)
///        └─random_repo (.git)
pub fn create_repos_tree3(path: &PathBuf) {
    // set root git init
    create_repos_tree2(path);

    let remotes = vec![
        "https://github.com/imgui-rs/imgui-rs.git",
        "https://github.com/console-rs/indicatif.git",
        "https://github.com/orf/git-workspace.git",
        "https://github.com/hakoerber/git-repo-manager.git",
    ];

    // get all dir
    for it in std::fs::read_dir(path).unwrap() {
        let dir_entry = match it {
            Ok(dir) => dir,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let sub_path = path.join(dir_entry.path());
        let mut rng = rand::thread_rng();

        // init repo randomly
        for idx in 0..rng.gen_range(0..=remotes.len()) {
            let pos = remotes[idx].rfind("/").unwrap();
            let (_, repo_name) = remotes[idx].split_at(pos + 1);
            let dir = sub_path.join(&repo_name);
            println!("{:?}", dir);
            std::fs::create_dir_all(dir.to_path_buf()).unwrap();
            // create local git repositoris
            excute_cmd(&dir, "git", &["init"]);

            // add remote
            excute_cmd(&dir, "git", &["remote", "add", "origin", remotes[idx]]);
        }
    }
}
