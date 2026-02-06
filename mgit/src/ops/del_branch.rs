use std::path::{Path, PathBuf};

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::git;
use crate::utils::current_dir;
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
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => current_dir(),
        };
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            branch,
            ignore,
        }
    }
}

pub async fn del_remote_branch(options: DelBranchOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let branch = options.branch;
    let mut ignore = options.ignore.unwrap_or_default();

    tracing::info!("Delete remote branch:");
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
            source: std::io::Error::other("Failed to load config"),
        });
    };

    let repo_configs = if let Some(repos) = mgit_config.repos.as_mut() {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to delete remote branch"));
    };

    if ignore.contains(&".".to_string()) {
        ignore.push("".to_string());
    }

    let mut errors = Vec::new();
    for repo_config in repo_configs.iter_mut() {
        let Some(local) = repo_config.local.as_ref() else {
            continue;
        };

        // only support new branch from exsit branch
        if repo_config.branch.is_none() {
            continue;
        }

        if ignore.contains(local) {
            continue;
        }

        // Safe to unwrap now as we checked above
        let rel_path = local;
        let full_path = Path::new(path).join(rel_path);

        match git::check_remote_branch_exist(&full_path, &branch).await {
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

        if let Err(e) = git::del_remote_branch(&full_path, &branch).await {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        let rel_path_display = Path::new(rel_path).display_path();
        let msg = StyleMessage::git_del_branch(rel_path_display, format!("origin/{}", branch));
        tracing::info!(message = %msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("del-remote-branch", errors.len());
        // OpsErrors struct is gone, construct string or specific error
        return Err(crate::error::MgitError::OpsError {
            message: format!("{}\nErrors:\n{:?}", msg, errors),
        });
    }

    let msg = StyleMessage::ops_success("del-remote-branch");
    Ok(msg)
}
