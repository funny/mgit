//! git init test
//! use following command:
//!     cargo test --test Init_test -- cli_init_no_args
//!     cargo test --test Init_test -- cli_init_force
//!     cargo test --test Init_test -- cli_init_with_path
//!     cargo test --test Init_test -- cli_init_with_path_force
//! or:
//!     cargo test --test Init_test -- cli_init_no_args --show-output
//!     cargo test --test Init_test -- cli_init_force --show-output
//!     cargo test --test Init_test -- cli_init_with_path --show-output
//!     cargo test --test Init_test -- cli_init_with_path_force --show-output

use assert_cmd::prelude::*;
use std::env;
use std::process::Command;

#[test]
fn cli_init_no_args() {
    Command::cargo_bin("mgit")
        .unwrap()
        .args(&["init"])
        .assert()
        .success();
}

#[test]
fn cli_init_force() {
    Command::cargo_bin("mgit")
        .unwrap()
        .args(&["init", "--force"])
        .assert()
        .success();
}

#[test]
fn cli_init_with_path() {
    let path = env::current_dir().unwrap().join("test_repo");
    std::fs::create_dir_all(path.clone()).unwrap();
    let path = path.into_os_string().into_string().unwrap();

    Command::cargo_bin("mgit")
        .unwrap()
        .args(&["init", &path])
        .assert()
        .success();
}

#[test]
fn cli_init_with_path_force() {
    let path = env::current_dir().unwrap().join("test_repo");
    std::fs::create_dir_all(path.clone()).unwrap();
    let path = path.into_os_string().into_string().unwrap();

    Command::cargo_bin("mgit")
        .unwrap()
        .args(&["init", &path, "--force"])
        .assert()
        .success();
}
