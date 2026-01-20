use std::path::Path;

use crate::error::MgitResult;
use crate::utils::cmd::exec_cmd;
use crate::utils::style_message::StyleMessage;

#[allow(dead_code)]
pub async fn has_authenticity(path: impl AsRef<Path>) -> MgitResult<String> {
    exec_cmd(path, "git", &["ls-remote"]).await
}

pub async fn is_remote_ref_valid(
    path: impl AsRef<Path>,
    remote_ref: impl AsRef<str>,
) -> MgitResult<()> {
    let remote_ref = remote_ref.as_ref();
    let args = ["branch", "--contains", remote_ref, "-r"];
    match exec_cmd(path, "git", &args).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let msg = StyleMessage::git_remote_not_found(remote_ref).to_string();
            Err(crate::error::MgitError::OpsError { message: msg })
        }
    }
}

pub async fn find_remote_name_by_url(
    path: impl AsRef<Path>,
    url: impl AsRef<str>,
) -> MgitResult<String> {
    crate::git::repo::is_repository(&path).await?;

    let url = url.as_ref();
    let args = ["remote", "-v"];
    let output = exec_cmd(&path, "git", &args).await?;

    for line in output.trim().lines() {
        if line.contains(url) {
            if let Some(remote_name) = line.split(url).next() {
                return Ok(remote_name.trim().to_string());
            }
        }
    }

    let msg = StyleMessage::git_remote_not_found(url).to_string();
    Err(crate::error::MgitError::OpsError { message: msg })
}

pub async fn find_remote_url_by_name(
    path: impl AsRef<Path>,
    name: impl AsRef<str>,
) -> MgitResult<String> {
    crate::git::repo::is_repository(&path).await?;

    let name = name.as_ref();
    let args = ["remote", "get-url", name];
    let output = exec_cmd(&path, "git", &args).await?;

    if let Some(remote_url) = output.trim().lines().next() {
        return Ok(remote_url.trim().to_string());
    }

    let msg = StyleMessage::git_remote_not_found(name).to_string();
    Err(crate::error::MgitError::OpsError { message: msg })
}

pub async fn add_remote_url(path: impl AsRef<Path>, url: impl AsRef<str>) -> MgitResult<()> {
    let args = ["remote", "add", "origin", url.as_ref()];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn update_remote_url(path: impl AsRef<Path>, url: impl AsRef<str>) -> MgitResult<String> {
    let args = ["remote", "set-url", "origin", url.as_ref()];
    exec_cmd(path, "git", &args).await
}

pub async fn get_remote_branches(path: impl AsRef<Path>) -> MgitResult<Vec<String>> {
    let path = path.as_ref();
    crate::git::repo::is_repository(path).await?;

    let args = ["branch", "-r"];
    let output = exec_cmd(path, "git", &args).await?;

    let mut branches = Vec::new();
    for file in output.trim().lines() {
        let line = file.trim();
        if line.contains("->") {
            continue;
        }
        let branch = match line.split_once('/') {
            Some((_remote, name)) => name.to_string(),
            None => line.to_string(),
        };
        branches.push(branch);
    }
    branches.sort();
    branches.dedup();
    Ok(branches)
}

pub async fn new_remote_branch(
    path: impl AsRef<Path>,
    base_branch: &str,
    new_branch: &str,
) -> MgitResult<()> {
    let arg = format!("origin/{}:refs/heads/{}", base_branch, new_branch);
    let args = vec!["push", "origin", arg.as_str(), "--force"];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn del_remote_branch(path: impl AsRef<Path>, branch: &str) -> MgitResult<()> {
    let args = vec!["push", "origin", "--delete", branch];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn check_remote_branch_exist(path: impl AsRef<Path>, branch: &str) -> MgitResult<bool> {
    let head = format!("refs/heads/{}", branch);
    let args = vec!["ls-remote", "--heads", "origin", head.as_str()];
    let output = exec_cmd(path, "git", &args).await?;
    Ok(output.contains(&head))
}

pub async fn new_local_tag(path: impl AsRef<Path>, local_ref: &str, tag: &str) -> MgitResult<()> {
    let mut args = vec!["tag", tag, "--force"];
    if !local_ref.is_empty() {
        args.push(local_ref);
    }

    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn push_tag(path: impl AsRef<Path>, tag: &str) -> MgitResult<()> {
    let args = vec!["push", "origin", tag, "--force"];
    exec_cmd(path, "git", &args).await.map(|_| ())
}
