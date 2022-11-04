use assert_cmd::prelude::*;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

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
