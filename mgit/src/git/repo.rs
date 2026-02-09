use std::path::Path;

use crate::error::MgitResult;
use crate::utils::cmd::exec_cmd;
use crate::utils::style_message::StyleMessage;

pub async fn is_repository(path: impl AsRef<Path>) -> MgitResult<()> {
    if path.as_ref().join(".git").is_dir() {
        let args = ["rev-parse", "--show-cdup"];
        if let Ok(output) = exec_cmd(path, "git", &args).await {
            if output.trim().is_empty() {
                return Ok(());
            }
        }
    }

    Err(crate::error::MgitError::OpsError {
        message: "repository not found!".to_string(),
    })
}

pub async fn get_current_commit(path: impl AsRef<Path>) -> MgitResult<String> {
    is_repository(&path).await?;
    let args = ["rev-parse", "HEAD"];
    let output = exec_cmd(path, "git", &args).await?;

    if let Some(oid) = output.trim().lines().next() {
        return Ok(oid.to_string());
    }

    Err(crate::error::MgitError::OpsError {
        message: "current commit not found.".to_string(),
    })
}

pub async fn get_tracking_branch(path: impl AsRef<Path>) -> MgitResult<String> {
    is_repository(&path).await?;
    let args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];

    let output = exec_cmd(path, "git", &args).await?;
    if !output.trim().is_empty() {
        return Ok(output.trim().to_string());
    }

    Err(crate::error::MgitError::OpsError {
        message: "untracked.".to_string(),
    })
}

pub async fn get_head_tags(path: impl AsRef<Path>) -> MgitResult<Vec<String>> {
    is_repository(&path).await?;
    let args = ["tag", "--points-at", "HEAD"];

    let output = exec_cmd(path, "git", &args).await?;

    if output.contains("fatal:") {
        return Err(crate::error::MgitError::OpsError { message: output });
    }

    let mut tags = Vec::new();
    for line in output.trim().lines() {
        tags.push(line.to_string());
    }

    Ok(tags)
}

pub async fn get_current_branch(path: impl AsRef<Path>) -> MgitResult<String> {
    is_repository(&path).await?;
    let args = ["branch", "--show-current"];
    let output = exec_cmd(&path, "git", &args).await?;

    for line in output.trim().lines() {
        let branch = line.to_string();
        let branch_output = exec_cmd(&path, "git", &["branch", "-l", &branch]).await?;
        if branch_output.contains(&branch) {
            return Ok(branch);
        }
    }
    Err(crate::error::MgitError::OpsError {
        message: "current branch not found.".to_string(),
    })
}

pub async fn get_branch_log(path: impl AsRef<Path>, branch: String) -> String {
    let args = ["show-branch", "--sha1-name", &branch];
    let output = exec_cmd(path, "git", &args).await.unwrap_or_default();
    output.trim().to_string()
}

pub async fn init(path: impl AsRef<Path>) -> MgitResult<()> {
    let args = ["init", "-b", "master"];
    exec_cmd(path, "git", &args).await.map(|_| ())
}

pub async fn local_branch_already_exist(
    path: impl AsRef<Path>,
    branch: impl AsRef<str>,
) -> MgitResult<bool> {
    let args = ["branch", "-l", branch.as_ref()];

    let output = exec_cmd(path, "git", &args).await?;
    let exist = output.trim().contains(branch.as_ref());
    Ok(exist)
}

pub async fn checkout(path: impl AsRef<Path>, args: &[&str]) -> MgitResult<()> {
    exec_cmd(path, "git", args).await.map(|_| ())
}

pub async fn set_tracking_remote_branch(
    full_path: impl AsRef<Path>,
    rel_path: impl AsRef<str>,
    local_branch: impl AsRef<str>,
    remote_ref: impl AsRef<str>,
    remote_desc: impl AsRef<str>,
) -> MgitResult<StyleMessage> {
    let args = ["branch", "--set-upstream-to", remote_ref.as_ref()];

    let msg = match exec_cmd(full_path, "git", &args).await {
        Ok(_) => StyleMessage::git_tracking_succ(rel_path, local_branch, remote_desc),
        Err(_) => StyleMessage::git_tracking_failed(rel_path, remote_desc),
    };
    Ok(msg)
}

pub async fn ls_files(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = ["ls-files", "-s"];
    exec_cmd(path, "git", &args).await
}

pub async fn log_current(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = [
        "log",
        "-1",
        "--pretty=format:\"%H%n%an <%ae>%n%ad%n%s%n\"",
        "--date=format-local:\"%Y-%m-%d %H:%M:%S\"",
    ];
    exec_cmd(path, "git", &args).await
}
