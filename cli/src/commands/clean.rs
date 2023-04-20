use clap::ArgMatches;
use globset::GlobBuilder;
use std::{
    env,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    config::repos::{load_config, TomlConfig},
    utils::logger,
};

pub(crate) fn exec(args: &ArgMatches) {
    // get input path
    let input_path = match args.get_one::<String>("path") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    // starting clean repos
    logger::new("Clean Status:");

    // if directory doesn't exist, finsh clean
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

    inner_exec(input_path, &toml_config);
}

fn inner_exec(input_path: impl AsRef<Path>, toml_config: &TomlConfig) {
    exec_clean(input_path, toml_config);
}

pub(crate) fn exec_clean(input_path: impl AsRef<Path>, toml_config: &TomlConfig) {
    let Some(toml_repos) = &toml_config.repos else {
        return;
    };

    let config_repo_paths: Vec<PathBuf> = toml_repos
        .into_iter()
        .map(|item| item.local.as_ref().unwrap())
        .map(|str| PathBuf::from(str))
        .collect();

    // search for git repos and create .gitrepos file
    let glob = GlobBuilder::new("**/.git")
        .literal_separator(true)
        .build()
        .unwrap()
        .compile_matcher();

    let mut it = WalkDir::new(&input_path).into_iter();
    let mut unused_paths: Vec<PathBuf> = Vec::new();

    // scan unused repositories
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => panic!("ERROR: {}", err),
            Some(Ok(entry)) => entry,
        };
        let path = entry.path();

        if glob.is_match(path) {
            // get relative path
            let mut pb = path.to_path_buf();
            pb.pop();
            let rel_path = pb.strip_prefix(&input_path).unwrap().to_path_buf();

            if !config_repo_paths.contains(&rel_path) {
                unused_paths.push(rel_path);
            }

            // just skip go into .git/ folder and continue
            it.skip_current_dir();
            continue;
        }
    }

    // remvoe unused repositories
    let mut count: u32 = 0;
    for unused_path in unused_paths {
        // find contianed repo path
        let contained_paths = find_contained_paths(&unused_path, &config_repo_paths);

        // remove unused directory
        if contained_paths.len() > 0 {
            if let Err(e) = remove_unused_files(&input_path, &unused_path, &contained_paths) {
                logger::remvoe_file_failed(&unused_path, e);
            };
        } else {
            let _ = std::fs::remove_dir_all(input_path.as_ref().join(&unused_path));
        }
        count += 1;

        logger::remvoe_file_succ(&unused_path);
    }

    // show statistics info
    match count {
        0 => logger::remove_none_repo_succ(),
        1 => logger::remove_one_repo_succ(),
        n => logger::remove_multi_repos_succ(n),
    }
}

fn find_contained_paths(unused_path: &PathBuf, config_repo_paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut contained_paths: Vec<PathBuf> = Vec::new();

    for config_repo_path in config_repo_paths {
        // add contained paths
        if config_repo_path
            .as_path()
            .starts_with(unused_path.as_path())
        {
            contained_paths.push(config_repo_path.to_path_buf());
        }
    }

    contained_paths
}

fn remove_unused_files(
    base_path: impl AsRef<Path>,
    unused_path: impl AsRef<Path>,
    contained_paths: &Vec<PathBuf>,
) -> Result<(), anyhow::Error> {
    let full_path = base_path.as_ref().join(&unused_path);

    // forearch files/folders begin with unused path
    let mut it = WalkDir::new(&full_path).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => panic!("ERROR: {}", err),
            Some(Ok(entry)) => entry,
        };

        // get file/folder path
        let file_path = entry.path();
        let rel_path = file_path.strip_prefix(&base_path)?.to_path_buf();

        // if the path is contained path, skip the path
        if contained_paths.contains(&rel_path) {
            it.skip_current_dir();
        }
        // if the path is not the parent of contained path, continue
        else if file_path.is_dir() && find_contained_paths(&rel_path, contained_paths).is_empty()
        {
            std::fs::remove_dir_all(&file_path)?;
            it.skip_current_dir();
        }
        // otherwise, delete the file/folder
        else if file_path.is_file() {
            std::fs::remove_file(&file_path)?;
        }
    }
    Ok(())
}
