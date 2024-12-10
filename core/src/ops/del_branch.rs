use anyhow::anyhow;
use std::env;
use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::repos::TomlConfig;
use crate::utils::error::{MgitError, MgitResult, OpsErrors};
use crate::utils::logger;
use crate::utils::path::PathExtension;
use crate::utils::StyleMessage;

pub struct DelBranchOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub branch: String,
    pub ignore: Option<Vec<String>>,
}

impl DelBranchOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        branch: String,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            branch,
            ignore,
        }
    }
}

pub fn del_remote_branch(options: DelBranchOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let branch = options.branch;
    let mut ignore = options.ignore.unwrap_or_default();

    logger::info("Delete remote branch:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        return Err(anyhow!(MgitError::DirNotFound(
            StyleMessage::dir_not_found(path)
        )));
    }

    // check if .gitrepos exists
    if !config_path.is_file() {
        return Err(anyhow!(MgitError::ConfigFileNotFound(
            StyleMessage::config_file_not_found()
        )));
    }

    // load config file(like .gitrepos)
    let Some(mut toml_config) = TomlConfig::load(config_path) else {
        return Err(anyhow!(MgitError::LoadConfigFailed));
    };

    let Some(toml_repos) = toml_config.repos.as_mut() else {
        return Ok("No repos to delete remote branch".into());
    };

    if ignore.contains(&".".to_string()) {
        ignore.push("".to_string());
    }

    let mut errors = Vec::new();
    for toml_repo in toml_repos.iter_mut() {
        let Some(local) = toml_repo.local.as_ref() else {
            continue;
        };

        // only support new branch from exsit branch
        if toml_repo.branch.is_none() {
            continue;
        }

        if ignore.contains(local) {
            continue;
        }

        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = Path::new(path).join(rel_path);

        match git::check_remote_branch_exist(&full_path, &branch) {
            Err(e) => {
                let error = StyleMessage::git_error(rel_path, &e);
                errors.push(error);
                continue;
            }

            Ok(false) => {
                continue;
            }

            Ok(true) => {}
        }

        if let Err(e) = git::del_remote_branch(full_path, &branch) {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        let rel_path_display = Path::new(rel_path).display_path();
        let msg = StyleMessage::git_del_branch(rel_path_display, &format!("origin/{}", branch));
        logger::info(msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("del-remote-branch", errors.len());
        let e = anyhow!(MgitError::OpsError {
            prefix: msg,
            errors: OpsErrors(errors),
        });

        return Err(e);
    }

    let msg = StyleMessage::ops_success("del-remote-branch");
    Ok(msg)
}
