use anyhow::anyhow;
use std::env;
use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::repos::TomlConfig;
use crate::utils::error::{MgitError, MgitResult, OpsErrors};
use crate::utils::logger;
use crate::utils::path::PathExtension;
use crate::utils::StyleMessage;

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
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
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

pub fn new_tag(options: NewTagOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let new_tag = options.new_tag;
    let push = options.push;
    let mut ignore = options.ignore.unwrap_or_default();

    logger::info("New tag:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        let e = MgitError::DirNotFound(StyleMessage::dir_not_found(path));
        return Err(anyhow!(e));
    }

    // check if .gitrepos exists
    if !config_path.is_file() {
        let e = MgitError::ConfigFileNotFound(StyleMessage::config_file_not_found());
        return Err(anyhow!(e));
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

        if ignore.contains(local) {
            continue;
        }

        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = Path::new(path).join(rel_path);

        let target_ref = if let Some(commit) = toml_repo.branch.as_ref() {
            commit.clone()
        } else if let Some(tag) = toml_repo.tag.as_ref() {
            tag.clone()
        } else if let Some(branch) = toml_repo.branch.as_ref() {
            branch.clone()
        } else if let Some(branch) = toml_config.default_branch.as_ref() {
            branch.clone()
        } else {
            "develop".to_string()
        };

        if let Err(e) = git::new_local_tag(&full_path, &target_ref, &new_tag) {
            let error = StyleMessage::git_error(rel_path, &e);
            errors.push(error);
            continue;
        }

        if push {
            if let Err(e) = git::push_tag(path, &new_tag) {
                let error = StyleMessage::git_error(rel_path, &e);
                errors.push(error);
                continue;
            }
        }

        let rel_path_display = Path::new(rel_path).display_path();
        let msg = StyleMessage::git_new_tag(rel_path_display, &new_tag);
        logger::info(msg);
    }

    if !errors.is_empty() {
        let msg = StyleMessage::ops_failed("new-tag", errors.len());
        let e = anyhow!(MgitError::OpsError {
            prefix: msg,
            errors: OpsErrors(errors),
        });

        return Err(e);
    }

    let msg = StyleMessage::ops_success("new-tag");
    Ok(msg)
}
