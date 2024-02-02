use std::path::Path;

use crate::utils::cmd::exec_cmd;
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StashMode {
    Normal,
    Stash,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResetType {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RemoteRef {
    Commit(String),
    Tag(String),
    Branch(String),
}

pub fn is_repository(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    if path.as_ref().join(".git").is_dir() {
        let args = ["rev-parse", "--show-cdup"];
        if let Ok(output) = exec_cmd(path, "git", &args) {
            if output.trim().is_empty() {
                return Ok(());
            }
        }
    }

    Err(anyhow::anyhow!("repository not found!"))
}

#[allow(dead_code)]
pub fn has_authenticity(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    exec_cmd(path, "git", &["ls-remote"])
}

pub fn is_remote_ref_valid(
    path: impl AsRef<Path>,
    remote_ref: impl AsRef<str>,
) -> Result<(), anyhow::Error> {
    let remote_ref = remote_ref.as_ref();
    let args = ["branch", "--contains", remote_ref, "-r"];
    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!(StyleMessage::git_remote_not_found(
            remote_ref
        ))),
    }
}

pub fn find_remote_name_by_url(
    path: impl AsRef<Path>,
    url: impl AsRef<str>,
) -> Result<String, anyhow::Error> {
    is_repository(&path)?;

    let url = url.as_ref();
    let args = ["remote", "-v"];
    let output = exec_cmd(path, "git", &args)?;

    for line in output.trim().lines() {
        if line.contains(url) {
            if let Some(remote_name) = line.split(url).next() {
                return Ok(remote_name.trim().to_string());
            }
        }
    }

    Err(anyhow::anyhow!(StyleMessage::git_remote_not_found(url)))
}

pub fn find_remote_url_by_name(
    path: impl AsRef<Path>,
    name: impl AsRef<str>,
) -> Result<String, anyhow::Error> {
    is_repository(&path)?;

    let name = name.as_ref();
    let args = ["remote", "get-url", name];
    let output = exec_cmd(path, "git", &args)?;

    if let Some(remote_url) = output.trim().lines().next() {
        return Ok(remote_url.trim().to_string());
    }

    Err(anyhow::anyhow!(StyleMessage::git_remote_not_found(name)))
}

pub fn get_current_commit(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    is_repository(&path)?;
    let args = ["rev-parse", "HEAD"];
    let output = exec_cmd(path, "git", &args)?;

    if let Some(oid) = output.trim().lines().next() {
        return Ok(oid.to_string());
    }

    Err(anyhow::anyhow!("current commit not found."))
}

pub fn get_tracking_branch(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    is_repository(&path)?;
    let args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];

    let output = exec_cmd(path, "git", &args)?;
    if !output.trim().is_empty() {
        return Ok(output.trim().to_string());
    }

    Err(anyhow::anyhow!("untracked."))
}

pub fn get_current_branch(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    is_repository(&path)?;
    let args = ["branch", "--show-current"];
    let output = exec_cmd(&path, "git", &args)?;

    for line in output.trim().lines() {
        let branch = line.to_string();
        // check if th branch exists
        let branch_output = exec_cmd(&path, "git", &["branch", "-l", &branch])?;
        if branch_output.contains(&branch) {
            return Ok(branch);
        }
    }
    Err(anyhow::anyhow!("current branch not found."))
}

pub fn get_branch_log(path: impl AsRef<Path>, branch: String) -> String {
    let args = ["show-branch", "--sha1-name", &branch];
    let output = exec_cmd(path, "git", &args).unwrap_or(String::new());
    output.trim().to_string()
}

pub fn get_untrack_files(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["ls-files", ".", "--exclude-standard", "--others"];
    exec_cmd(path, "git", &args)
}

pub fn get_changed_files(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["diff", "--name-only"];
    exec_cmd(path, "git", &args)
}

pub fn get_staged_files(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["diff", "--cached", "--name-only"];
    exec_cmd(path, "git", &args)
}

pub fn get_rev_list_count(
    path: impl AsRef<Path>,
    branch_pair: impl AsRef<str>,
) -> Result<String, anyhow::Error> {
    let args = ["rev-list", "--count", "--left-right", branch_pair.as_ref()];
    exec_cmd(path, "git", &args)
}

// 由于不同平台、不同用户的全局git config配置会有不同的git init [defaultBranch]
// 可能是main、master又或者用户自定义的
// 此处固定将git init的初始分支命名为master，以避免产生歧义
pub fn init(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let args = ["init", "-b", "master"];
    exec_cmd(path, "git", &args).map(|_| ())
}

pub fn add_remote_url(path: impl AsRef<Path>, url: impl AsRef<str>) -> anyhow::Result<()> {
    // git remote add origin {url}
    let args = ["remote", "add", "origin", url.as_ref()];
    exec_cmd(path, "git", &args).map(|_| ())
}

pub fn clean(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let args = ["clean", "-fd"];
    exec_cmd(path, "git", &args).map(|_| ())
}

pub fn reset(
    path: impl AsRef<Path>,
    reset_type: impl AsRef<str>,
    remote_ref: impl AsRef<str>,
) -> anyhow::Result<()> {
    let args = ["reset", reset_type.as_ref(), remote_ref.as_ref()];

    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

pub fn stash(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["stash", "--include-untracked"];
    exec_cmd(path, "git", &args)
}

pub fn stash_pop(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["stash", "pop"];
    exec_cmd(path, "git", &args)
}

pub fn local_branch_already_exist(
    path: impl AsRef<Path>,
    branch: impl AsRef<str>,
) -> Result<bool, anyhow::Error> {
    let args = ["branch", "-l", branch.as_ref()];

    let output = exec_cmd(path, "git", &args)?;
    let exist = output.trim().contains(branch.as_ref());
    Ok(exist)
}

pub fn checkout(path: impl AsRef<Path>, args: &[&str]) -> anyhow::Result<()> {
    exec_cmd(path, "git", args).map(|_| ())
}

#[allow(dead_code)]
pub fn get_remote_branches(path: impl AsRef<Path>) -> Vec<String> {
    let mut branches = Vec::new();
    let args = ["branch", "-r"];

    if let Ok(output) = exec_cmd(path, "git", &args) {
        for file in output.trim().lines() {
            let branch = file.trim().replace("origin/", "");
            branches.push(branch);
        }
    }
    branches
}

/// git branch --set-upstream-to <name>, true only when remote head is branch
pub fn set_tracking_remote_branch(
    full_path: impl AsRef<Path>,
    rel_path: impl AsRef<str>,
    local_branch: impl AsRef<str>,
    remote_ref: impl AsRef<str>,
    remote_desc: impl AsRef<str>,
) -> Result<StyleMessage, anyhow::Error> {
    let args = ["branch", "--set-upstream-to", remote_ref.as_ref()];

    let msg = match exec_cmd(full_path, "git", &args) {
        Ok(_) => StyleMessage::git_tracking_succ(rel_path, local_branch, remote_desc),
        Err(_) => StyleMessage::git_tracking_failed(rel_path, remote_desc),
    };
    Ok(msg)
}

pub fn update_remote_url(
    path: impl AsRef<Path>,
    url: impl AsRef<str>,
) -> Result<String, anyhow::Error> {
    let args = ["remote", "set-url", "origin", url.as_ref()];
    exec_cmd(path, "git", &args)
}

pub fn ls_files(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = ["ls-files", "-s"];
    exec_cmd(path, "git", &args)
}

pub fn log_current(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let args = [
        "log",
        "-1",
        "--pretty=format:\"%H%n%an <%ae>%n%ad%n%s%n\"",
        "--date=format:\"%Y-%m-%d %H:%M:%S\"",
    ];
    exec_cmd(path, "git", &args)
}

pub fn sparse_checkout_set(path: impl AsRef<Path>, dirs: &Vec<String>) -> Result<(), anyhow::Error> {
    let mut args = vec!["sparse-checkout", "set", "--no-cone"];
    for dir in dirs{
        args.push(dir)
    }

    exec_cmd(path, "git", &args).map(|_| ())
}

pub fn sparse_checkout_disable(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let args = vec!["sparse-checkout", "disable"];
    exec_cmd(path, "git", &args).map(|_| ())
}
