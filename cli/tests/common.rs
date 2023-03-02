use assert_cmd::prelude::*;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[allow(unused)]
pub mod failed_message {
    pub const GIT_INIT: &str = "git init failed";
    pub const GIT_ADD_REMOTE: &str = "git add remote failed";
    pub const GIT_STAGE: &str = "git stage failed";
    pub const GIT_COMMIT: &str = "git commit failed";
    pub const GIT_STATUS: &str = "git status failed";
    pub const GIT_CHECKOUT: &str = "git checkout failed";
    pub const GIT_RESET: &str = "git reset failed";
    pub const GIT_STASH_LIST: &str = "git stash list failed";
    pub const GIT_STASH_POP: &str = "git stash pop failed";
    pub const GIT_BRANCH: &str = "git branch failed";
    pub const GIT_FETCH: &str = "git fetch failed";
    pub const GIT_CONFIG: &str = "git config failed";

    pub const WRITE_FILE: &str = "write file failed";
}

pub fn execute_cmd(path: &PathBuf, cmd: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let output = std::process::Command::new(cmd)
        .current_dir(path.to_path_buf())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    match output.status.success() {
        false => Err(anyhow::anyhow!(stderr)),
        true => Ok(stdout),
    }
}

pub fn execute_cargo_cmd(cmd: &str, args: &[&str]) {
    Command::cargo_bin(cmd)
        .unwrap()
        .args(args)
        .assert()
        .success();
}
