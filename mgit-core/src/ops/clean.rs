use globset::GlobBuilder;
use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::repos::load_config;
use crate::utils::logger;

pub struct CleanOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
}

impl CleanOptions {
    pub fn new(path: Option<impl AsRef<Path>>, config_path: Option<impl AsRef<Path>>) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self { path, config_path }
    }
}

pub fn clean_repo(options: CleanOptions) {
    let path = &options.path;
    let config_path = &options.config_path;

    // starting clean repos
    logger::new("Clean Status:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        logger::dir_not_found(&path);
        return;
    }
    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::config_file_not_found();
        return;
    }
    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(&config_path) else{
        logger::new("load config file failed!");
        return;
    };

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

    let input_path = path.to_owned();
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
            let _ = std::fs::remove_dir_all(input_path.join(&unused_path));
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
