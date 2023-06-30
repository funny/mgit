use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::repos::load_config;
use crate::ops::CleanOptions;
use crate::utils::logger;
use crate::utils::path::PathExtension;
use crate::utils::style_message::StyleMessage;

pub struct ListFilesOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
}

impl ListFilesOptions {
    pub fn new(path: Option<impl AsRef<Path>>, config_path: Option<impl AsRef<Path>>) -> Self {
        let clean_options = CleanOptions::new(path, config_path);
        Self {
            path: clean_options.path,
            config_path: clean_options.config_path,
        }
    }
}

pub fn list_files(options: ListFilesOptions) {
    let path = &options.path;
    let config_path = &options.config_path;

    // if directory doesn't exist, return
    if !path.is_dir() {
        logger::error(StyleMessage::dir_not_found(path));
        return;
    }

    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::error(StyleMessage::config_file_not_found());
        return;
    }

    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(config_path) else{
        logger::error("load config file failed!");
        return;
    };

    let Some( toml_repos) = toml_config.repos else {
        return;
    };

    for toml_repo in &toml_repos {
        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = path.join(rel_path);
        let Ok(content) = git::ls_files(&full_path) else {
            continue;
        };

        for line in content.trim().lines() {
            if let Some((left, right)) = line.rsplit_once("\t") {
                let split_str = match !rel_path.ends_with("\\") && !rel_path.ends_with("/") {
                    true => "/",
                    false => "",
                };

                let path = format!("{}{}{}", rel_path, split_str, right);
                let path = path.norm_path();
                println!("{}\t{}", left, path);
            }
        }
    }
}
