use std::path::Path;

use crate::error::MgitResult;
use crate::utils::cmd::exec_cmd;

pub async fn clean(path: impl AsRef<Path>) -> MgitResult<()> {
    let args = ["clean", "-fd"];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn reset(
    path: impl AsRef<Path>,
    reset_type: impl AsRef<str>,
    remote_ref: impl AsRef<str>,
) -> MgitResult<()> {
    let args = ["reset", reset_type.as_ref(), remote_ref.as_ref()];

    match exec_cmd(path, "git", &args).await {
        Ok(_) => Ok(()),
        Err(e) => Err(crate::error::MgitError::OpsError {
            message: format!("Error: {}", e),
        }),
    }
}

pub async fn add_untracked_files(path: impl AsRef<Path>) -> MgitResult<String> {
    let path = path.as_ref();
    let args = ["ls-files", "-o", "--exclude-standard"];
    let paths_desc = exec_cmd(path, "git", &args).await?;
    if paths_desc.is_empty() {
        return Ok("not found any unchecked file to add".to_string());
    }

    let mut args = vec!["add"];

    for file in paths_desc.trim().split('\n') {
        args.push(file);
    }

    exec_cmd(path, "git", &args).await
}

pub async fn stash(path: impl AsRef<Path>) -> MgitResult<String> {
    let path = path.as_ref();

    add_untracked_files(path).await?;

    let args = ["stash", "-u"];
    exec_cmd(path, "git", &args).await
}

pub async fn stash_pop(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = ["stash", "pop"];
    exec_cmd(path, "git", &args).await
}

pub async fn sparse_checkout_set(path: impl AsRef<Path>, dirs: &Vec<String>) -> MgitResult<()> {
    let mut args = vec!["sparse-checkout", "set", "--no-cone"];
    for dir in dirs {
        args.push(dir);
    }

    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn sparse_checkout_disable(path: impl AsRef<Path>) -> MgitResult<()> {
    let args = vec!["sparse-checkout", "disable"];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn sparse_checkout_list(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = vec!["sparse-checkout", "list"];
    exec_cmd(path, "git", &args).await
}
