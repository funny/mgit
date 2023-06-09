use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::repos::load_config;
use crate::utils::{logger, path::norm_path};

pub trait ListFilesOptions {
    fn new_list_files_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
    ) -> Self;

    fn path(&self) -> &PathBuf;
    fn config_path(&self) -> &PathBuf;
}

pub fn list_files(options: impl ListFilesOptions) {
    let path = options.path();
    let config_path = options.config_path();

    // if directory doesn't exist, return
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

    let Some( toml_repos) = toml_config.repos else {
        return;
    };

    for toml_repo in &toml_repos {
        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = path.join(rel_path);

        if let Ok(res) = git::ls_files(&full_path) {
            for line in res.trim().lines() {
                if let Some((left, right)) = line.rsplit_once("\t") {
                    let split_str = match !rel_path.ends_with("\\") && !rel_path.ends_with("/") {
                        true => "/",
                        false => "",
                    };

                    let path = format!("{}{}{}", rel_path, split_str, right);
                    let path = norm_path(path);
                    println!("{}\t{}", left, path);
                }
            }
        }
    }
}
