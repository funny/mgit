use crate::core::git;
use crate::core::repo::{exclude_ignore, TomlRepo};
use crate::core::repos::load_config;
use std::path::{Path, PathBuf};

use crate::utils::logger;

use super::RemoteRef;

pub trait TrackOptions {
    fn new_track_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        ignore: Option<Vec<String>>,
    ) -> Self;

    fn path(&self) -> &PathBuf;
    fn config_path(&self) -> &PathBuf;
    fn ignore(&self) -> Option<&Vec<String>>;
}

pub fn track(options: impl TrackOptions) {
    let path = options.path();
    let config_path = options.config_path();
    let ignore = options.ignore();

    // starting clean repos
    logger::new("Track status:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        logger::dir_not_found(path);
        return;
    }
    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::config_file_not_found();
        return;
    }
    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(config_path) else{
        logger::new("load config file failed!");
        return;
    };

    // handle track
    let Some(mut toml_repos) = toml_config.repos else {
        return;
    };

    let default_branch = toml_config.default_branch;

    // ignore specified repositories
    exclude_ignore(
        &mut toml_repos,
        ignore.map(|it| it.iter().collect::<Vec<&String>>()),
    );

    for toml_repo in &toml_repos {
        if let Ok(res) = set_tracking_remote_branch(path, toml_repo, &default_branch) {
            logger::new(format!("  {}", res));
        }
    }
}

pub(crate) fn set_tracking_remote_branch(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    default_branch: &Option<String>,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);

    // get local current branch
    let local_branch = git::get_current_branch(full_path.as_path())?;

    let mut toml_repo = toml_repo.to_owned();
    // use default branch when branch is null
    if toml_repo.branch.is_none() {
        toml_repo.branch = default_branch.to_owned();
    }

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };
    let remote_desc = match remote_ref {
        RemoteRef::Commit(commit) => commit[..7].to_string(),
        RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    if toml_repo.commit.is_some() || toml_repo.tag.is_some() {
        let res = logger::fmt_untrack_desc(rel_path, &remote_desc);
        return Ok(res);
    }

    git::set_tracking_remote_branch(
        full_path,
        rel_path,
        local_branch,
        remote_ref_str,
        remote_desc,
    )
}
