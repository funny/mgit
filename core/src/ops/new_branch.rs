use anyhow::anyhow;
use std::env;
use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::repos::TomlConfig;
use crate::utils::error::{MgitError, MgitResult, OpsErrors};
use crate::utils::logger;
use crate::utils::path::PathExtension;
use crate::utils::StyleMessage;

pub struct NewBranchOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub new_config_path: Option<PathBuf>,
    pub new_branch: String,
    pub force: bool,
    pub ignore: Option<Vec<String>>,
}

impl NewBranchOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        new_config_path: Option<PathBuf>,
        new_branch: String,
        force: bool,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            new_config_path,
            new_branch,
            force,
            ignore,
        }
    }
}

pub fn new_remote_branch(options: NewBranchOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let new_branch = options.new_branch;
    let new_config_path = options.new_config_path;
    let force = options.force;
    let mut ignore = options.ignore.unwrap_or_default();

    logger::info("New remote branch:");
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
        return Ok("No repos to create new remote branch".into());
    };

    if ignore.contains(&".".to_string()) {
        ignore.push("".to_string());
    }

    let mut errors = Vec::new();
    for toml_repo in toml_repos.iter_mut() {
        let Some(local) = toml_repo.local.as_ref() else {
            continue;
        };

        // only support new branch from exist branch
        if toml_repo.branch.is_none() {
            continue;
        }

        if ignore.contains(local) {
            continue;
        }

        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = Path::new(path).join(rel_path);
        let base_branch = toml_repo.branch.as_ref().unwrap();

        if !force {
            match git::check_remote_branch_exist(&full_path, &new_branch) {
                Err(e) => {
                    let error = StyleMessage::git_error(rel_path, &e);
                    errors.push(error);
                    continue;
                }

                Ok(true) => {
                    let e: anyhow::Error =
                        anyhow!("origin/{} already exist, try force mode again", &new_branch);
                    let error = StyleMessage::git_error(rel_path, &e);
                    errors.push(error);
                    continue;
                }

                Ok(false) => {}
            }
        }

        if let Err(e) = git::new_remote_branch(full_path, base_branch, &new_branch) {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        toml_repo.branch = Some(new_branch.clone());
        let rel_path_display = Path::new(rel_path).display_path();

        let msg = StyleMessage::git_new_branch(rel_path_display, &new_branch);
        logger::info(msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("new-remote-branch", errors.len());
        let e = anyhow!(MgitError::OpsError {
            prefix: msg,
            errors: OpsErrors(errors),
        });

        return Err(e);
    }

    if let Some(new_config_path) = new_config_path {
        let toml_string = toml_config.serialize();
        std::fs::write(new_config_path, toml_string).expect("Failed to write file .gitrepos!");
    }

    let msg = StyleMessage::ops_success("new-remote-branch");
    Ok(msg)
}
