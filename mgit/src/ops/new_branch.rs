use std::env;
use std::path::{Path, PathBuf};

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::git;
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

pub async fn new_remote_branch(options: NewBranchOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let new_branch = options.new_branch;
    let new_config_path = options.new_config_path;
    let force = options.force;
    let mut ignore = options.ignore.unwrap_or_default();

    tracing::info!("New remote branch:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        return Err(crate::error::MgitError::DirNotFound { path: path.clone() });
    }

    // check if .gitrepos exists
    if !config_path.is_file() {
        return Err(crate::error::MgitError::ConfigFileNotFound {
            path: config_path.clone(),
        });
    }

    // load config file(like .gitrepos)
    let Some(mut mgit_config) = MgitConfig::load(config_path) else {
        return Err(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "Failed to load config"),
        });
    };

    let repo_configs = if let Some(repos) = mgit_config.repos.as_mut() {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to create new remote branch"));
    };

    if ignore.contains(&".".to_string()) {
        ignore.push("".to_string());
    }

    let mut errors = Vec::new();
    for repo_config in repo_configs.iter_mut() {
        let Some(local) = repo_config.local.as_ref() else {
            continue;
        };

        // only support new branch from exist branch
        if repo_config.branch.is_none() {
            continue;
        }

        if ignore.contains(local) {
            continue;
        }

        let rel_path = repo_config.local.as_ref().unwrap();
        let full_path = Path::new(path).join(rel_path);
        let base_branch = repo_config.branch.as_ref().unwrap();

        if !force {
            match git::check_remote_branch_exist(&full_path, &new_branch).await {
                Err(e) => {
                    let error = StyleMessage::git_error(rel_path, &e);
                    errors.push(error);
                    continue;
                }

                Ok(true) => {
                    let e = format!("origin/{} already exist, try force mode again", &new_branch);
                    let error = StyleMessage::git_error_str(rel_path, &e);
                    errors.push(error);
                    continue;
                }

                Ok(false) => {}
            }
        }

        if let Err(e) = git::new_remote_branch(full_path, base_branch, &new_branch).await {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        repo_config.branch = Some(new_branch.clone());
        let rel_path_display = Path::new(rel_path).display_path();

        let msg = StyleMessage::git_new_branch(rel_path_display, &new_branch);
        tracing::info!(message = %msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("new-remote-branch", errors.len());
        return Err(crate::error::MgitError::OpsError {
            message: format!("{}\nErrors:\n{:?}", msg, errors),
        });
    }

    if let Some(new_config_path) = new_config_path {
        use crate::config::serialize_config;
        let toml_string = serialize_config(&mgit_config);
        tokio::fs::write(new_config_path, toml_string)
            .await
            .map_err(|_| crate::error::MgitError::OpsError {
                message: "Failed to write file .gitrepos".into(),
            })?;
    }

    let msg = StyleMessage::ops_success("new-remote-branch");
    Ok(msg)
}
