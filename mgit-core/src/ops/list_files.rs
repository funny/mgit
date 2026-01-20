use std::path::{Path, PathBuf};

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::git;
use crate::ops::CleanOptions;
use crate::utils::label;
use crate::utils::path::PathExtension;

pub struct ListFilesOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub labels: Option<Vec<String>>,
}

impl ListFilesOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        labels: Option<Vec<String>>,
    ) -> Self {
        let clean_options = CleanOptions::new(path, config_path, labels);
        Self {
            path: clean_options.path,
            config_path: clean_options.config_path,
            labels: clean_options.labels,
        }
    }
}

pub async fn list_files(options: ListFilesOptions) -> MgitResult<Vec<String>> {
    let path = &options.path;
    let config_path = &options.config_path;

    // if directory doesn't exist, return
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
    let Some(mgit_config) = MgitConfig::load(config_path) else {
        return Err(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "Failed to load config"),
        });
    };

    let Some(mut repo_configs) = mgit_config.repos else {
        return Ok(vec![]);
    };

    if let Some(labels) = options.labels {
        repo_configs = label::filter(&repo_configs, &labels).cloned().collect();
    }

    let mut files = Vec::new();

    // To improve performance, we can process repos in parallel, but output order might matter
    // list_files usually needs deterministic output, so serial or gather-and-sort is needed.
    // For now, let's just make it async serial.

    for repo_config in repo_configs {
        let rel_path = repo_config.local.as_ref().unwrap();
        let full_path = path.join(rel_path);

        if let Ok(content) = git::ls_files(full_path).await {
            for line in content.trim().lines() {
                if let Some((left, right)) = line.rsplit_once('\t') {
                    let split_str = match !rel_path.ends_with('\\') && !rel_path.ends_with('/') {
                        true => "/",
                        false => "",
                    };

                    let path = format!("{}{}{}", rel_path, split_str, right);
                    let path = path.norm_path().trim_matches('/').to_string();
                    files.push(format!("{}\t{}", left, path));
                }
            }
        }
    }

    Ok(files)
}
