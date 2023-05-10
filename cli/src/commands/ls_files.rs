use clap::ArgMatches;
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    config::repos::{load_config, TomlConfig},
    git,
    utils::{logger, path::norm_path},
};

pub(crate) fn exec(args: &ArgMatches) {
    // get input path
    let input_path = match args.get_one::<String>("path") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    // if directory doesn't exist, return
    if !input_path.is_dir() {
        logger::dir_not_found(&input_path);
        return;
    }

    // set config file path
    let config_file = match args.get_one::<PathBuf>("config") {
        Some(r) => r.to_owned(),
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if !config_file.is_file() {
        logger::config_file_not_found();
        return;
    }

    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(&config_file) else{
        logger::new("load config file failed!");
        return;
    };

    inner_exec(input_path, toml_config);
}

fn inner_exec(input_path: impl AsRef<Path>, toml_config: TomlConfig) {
    let Some( toml_repos) = toml_config.repos else {
        return;
    };

    for toml_repo in &toml_repos {
        let rel_path = toml_repo.local.as_ref().unwrap();
        let full_path = input_path.as_ref().join(rel_path);

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
