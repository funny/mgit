use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

use crate::core::git;
use crate::core::git::RemoteRef;
use crate::utils::path::PathExtension;
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RepoId {
    pub id: usize,
    pub repo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlRepo {
    pub local: Option<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub commit: Option<String>,
}

impl RepoId {
    pub fn new(id: usize, repo: impl AsRef<str>) -> Self {
        Self {
            id,
            repo: repo.as_ref().to_string().replace(['/', '\\'], "_"),
        }
    }
}

impl TomlRepo {
    pub fn get_remote_name(&self, path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
        let remote_url = self
            .remote
            .as_ref()
            .with_context(|| "remote url is null.")?;
        git::find_remote_name_by_url(path, remote_url)
    }

    pub fn get_remote_ref(&self, path: &Path) -> Result<RemoteRef, anyhow::Error> {
        let remote_name = &self.get_remote_name(path)?;
        // priority: commit/tag/branch(default-branch)
        let remote_ref = {
            if let Some(commit) = &self.commit {
                RemoteRef::Commit(commit.to_string())
            } else if let Some(tag) = &self.tag {
                RemoteRef::Tag(tag.to_string())
            } else if let Some(branch) = &self.branch {
                let branch = format!("{}/{}", remote_name, branch);
                RemoteRef::Branch(branch)
            } else {
                return Err(anyhow::anyhow!("remote ref is invalid!"));
            }
        };
        Ok(remote_ref)
    }
}

pub fn exclude_ignore(toml_repos: &mut Vec<TomlRepo>, ignore: Option<Vec<&String>>) {
    if let Some(ignore_paths) = ignore {
        for ignore_path in ignore_paths {
            if let Some(idx) = toml_repos.iter().position(|r| {
                if let Some(rel_path) = r.local.as_ref() {
                    // consider "." as root path
                    rel_path.display_path() == *ignore_path
                } else {
                    false
                }
            }) {
                toml_repos.remove(idx);
            }
        }
    }
}

/// get full ahead/behind values between branches
pub fn cmp_local_remote(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    default_branch: &Option<String>,
    use_tracking_remote: bool,
) -> Result<StyleMessage, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);

    let mut toml_repo = toml_repo.to_owned();
    // use default branch when branch is null
    if toml_repo.branch.is_none() {
        toml_repo.branch = default_branch.to_owned();
    }

    // priority: commit/tag/branch(default-branch)
    let (remote_ref_str, remote_desc) = {
        if use_tracking_remote {
            let remote_ref_str = git::get_tracking_branch(&full_path)?;
            (remote_ref_str.clone(), remote_ref_str)
        } else {
            let remote_ref = toml_repo.get_remote_ref(&full_path)?;
            let remote_ref_str = match remote_ref.clone() {
                RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
            };
            let remote_desc = match remote_ref {
                RemoteRef::Commit(commit) => commit[..7].to_string(),
                RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
            };
            (remote_ref_str, remote_desc)
        }
    };

    // if specified remote commit/tag/branch is null
    if remote_desc.is_empty() {
        return Ok("not tracking".into());
    }

    let mut changed_files: HashSet<String> = HashSet::new();

    // get untracked files (uncommit)
    if let Ok(output) = git::get_untrack_files(&full_path) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    // get tracked and changed files (uncommit)
    if let Ok(output) = git::get_changed_files(&full_path) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    // get cached(staged) files (uncommit)
    if let Ok(output) = git::get_staged_files(&full_path) {
        for file in output.trim().lines() {
            changed_files.insert(file.to_string());
        }
    }

    let mut changes_desc: Option<StyleMessage> = None;
    if !changed_files.is_empty() {
        // format changes tooltip
        changes_desc = StyleMessage::git_changes(changed_files.len());
    }

    // get local branch
    let branch = git::get_current_branch(&full_path)?;

    if branch.is_empty() {
        return Ok("init commit".into());
    }

    // get rev-list between local branch and specified remote commit/tag/branch
    let branch_pair = format!("{}...{}", &branch, &remote_ref_str);
    let mut commit_desc: Option<StyleMessage> = None;
    if let Ok(output) = git::get_rev_list_count(&full_path, branch_pair) {
        let re = Regex::new(r"(\d+)\s*(\d+)").unwrap();

        if let Some(caps) = re.captures(&output) {
            // format commit tooltip
            let (ahead, behind) = (&caps[1], &caps[2]);
            commit_desc = StyleMessage::git_commits(ahead, behind);
        }
    } else {
        // if git rev-list find "unknown revision" error
        commit_desc = Some(StyleMessage::git_unknown_revision());
    }

    // show diff overview
    let desc = match (commit_desc, changes_desc) {
        (None, None) => {
            let branch_log = git::get_branch_log(&full_path, branch);
            StyleMessage::git_update_to_date(branch_log)
        }
        (commit_desc, changes_desc) => {
            StyleMessage::git_diff(remote_desc, commit_desc, changes_desc)
        }
    };

    Ok(desc)
}
