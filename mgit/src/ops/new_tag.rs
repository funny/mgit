use std::path::{Path, PathBuf};

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::git;
use crate::utils::current_dir;
use crate::utils::path::PathExtension;
use crate::utils::StyleMessage;

#[derive(Debug)]
pub struct NewTagOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub new_tag: String,
    pub push: bool,
    pub ignore: Option<Vec<String>>,
}

impl NewTagOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        new_tag: String,
        push: bool,
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
            new_tag,
            push,
            ignore,
        }
    }
}

pub async fn new_tag(options: NewTagOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let new_tag = options.new_tag;
    let push = options.push;
    let mut ignore = options.ignore.unwrap_or_default();

    tracing::info!("New tag:");
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

        if ignore.contains(local) {
            continue;
        }

        // Safe to unwrap now as we checked above
        let rel_path = local;
        let full_path = Path::new(path).join(rel_path);

        // NOTE: current head ref
        let target_ref = "";

        if let Err(e) = git::new_local_tag(&full_path, target_ref, &new_tag).await {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        if push {
            if let Err(e) = git::push_tag(path, &new_tag).await {
                let error = StyleMessage::git_error(rel_path, &e);
                errors.push(error);
                continue;
            }
        }

        let rel_path_display = Path::new(rel_path).display_path();
        let msg = StyleMessage::git_new_tag(rel_path_display, &new_tag);
        tracing::info!(message = %msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("new-tag", errors.len());
        return Err(crate::error::MgitError::OpsError {
            message: format!("{}\nErrors:\n{:?}", msg, errors),
        });
    }

    let msg = StyleMessage::ops_success("new-tag");
    Ok(msg)
}
