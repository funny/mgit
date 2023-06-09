use std::path::Path;

use crate::utils::{cmd::exec_cmd, logger};

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
        Err(_) => Err(anyhow::anyhow!(logger::fmt_remote_not_found(remote_ref))),
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

    Err(anyhow::anyhow!(logger::fmt_remote_not_found(url)))
}

pub fn find_remote_url_by_name(
    path: impl AsRef<Path>,
    name: impl AsRef<str>,
) -> Result<String, anyhow::Error> {
    is_repository(&path)?;

    let name = name.as_ref();
    let args = ["remote", "get-url", name];
    let output = exec_cmd(path, "git", &args)?;

    for remote_url in output.trim().lines() {
        return Ok(remote_url.trim().to_string());
    }

    Err(anyhow::anyhow!(logger::fmt_remote_not_found(name)))
}

pub fn get_current_commit(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    is_repository(&path)?;
    let args = ["rev-parse", "HEAD"];
    let output = exec_cmd(path, "git", &args)?;

    for oid in output.trim().lines() {
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

pub fn init(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let args = ["init"];
    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

pub fn add_remote_url(path: impl AsRef<Path>, url: impl AsRef<str>) -> anyhow::Result<()> {
    // git remote add origin {url}
    let args = ["remote", "add", "origin", url.as_ref()];

    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

pub fn clean(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let args = ["clean", "-fd"];

    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
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
    match exec_cmd(path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
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
) -> Result<String, anyhow::Error> {
    let args = ["branch", "--set-upstream-to", remote_ref.as_ref()];

    if exec_cmd(full_path, "git", &args).is_ok() {
        let res = logger::fmt_tracking_succ_desc(rel_path, local_branch, remote_desc);
        Ok(res)
    } else {
        let res = logger::fmt_tracking_failed_desc(rel_path, remote_desc);
        Ok(res)
    }
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
