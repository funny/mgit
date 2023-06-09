use globset::GlobBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::git;
use crate::core::repo::TomlRepo;
use crate::core::repos::TomlConfig;

use crate::ops::SnapshotType;
use crate::utils::logger;
use crate::utils::path::{display_path, norm_path};

pub trait SnapshotOptions {
    fn new_snapshot_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        force: Option<bool>,
        snapshot_type: Option<SnapshotType>,
        ignore: Option<Vec<String>>,
    ) -> Self;
    fn path(&self) -> &PathBuf;
    fn config_path(&self) -> &PathBuf;
    fn force(&self) -> bool;
    fn snapshot_type(&self) -> &SnapshotType;
    fn ignore(&self) -> Option<&Vec<String>>;
}

pub fn snapshot_repo(options: impl SnapshotOptions) {
    let path = options.path();
    let config_path = options.config_path();
    let force = options.force();
    let snapshot_type = options.snapshot_type();
    let ignore = options.ignore();

    // start taking snapshot repos
    logger::command_start("take snapshot", &path);

    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        logger::dir_not_found(path);
        return;
    }

    // check if .gitrepos exists
    if config_path.is_file() && !force {
        logger::dir_already_inited(path);
        return;
    }

    let mut toml_config = TomlConfig {
        version: None,
        default_branch: Some(String::from("develop")),
        default_remote: None,
        repos: None,
    };

    // search for git repos and create .gitrepos file
    let glob = GlobBuilder::new("**/.git")
        .literal_separator(true)
        .build()
        .unwrap()
        .compile_matcher();

    logger::new("search and add git repos:");
    let mut count = 0;
    let input_path = path.to_owned();
    let mut it = WalkDir::new(&input_path).into_iter();
    let mut repos: Vec<TomlRepo> = Vec::new();
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
            let rel_path = pb.strip_prefix(&input_path).unwrap();

            // normalize path if needed
            let norm_path = norm_path(&rel_path.to_str().unwrap().to_string());

            // if git in root path, represent it by "."
            let norm_str = display_path(&norm_path);

            // ignore specified path
            if let Some(ignore_paths) = ignore.as_ref() {
                if ignore_paths.contains(&&norm_str) {
                    continue;
                }
            }

            // check repository valid
            if git::is_repository(pb.as_path()).is_err() {
                logger::new(format!("Failed to open repo {}!", &norm_str));
                continue;
            }

            // get remote
            let remote = match git::find_remote_url_by_name(pb.as_path(), &"origin".to_string()) {
                Ok(r) => Some(r),
                _ => None,
            };

            let mut commit: Option<String> = None;
            let mut branch: Option<String> = None;

            // snapshot commit or remote-branch
            match snapshot_type {
                SnapshotType::Commit => {
                    // get local head commit id
                    if let Ok(oid) = git::get_current_commit(pb.as_path()) {
                        commit = Some(oid);
                    }
                }
                SnapshotType::Branch => {
                    // get tracking brach
                    if let Ok(refname) = git::get_tracking_branch(pb.as_path()) {
                        // split, like origin/master
                        if let Some((_, branch_ref)) = refname.split_once("/") {
                            branch = Some(branch_ref.trim().to_string());
                        }
                    }
                }
            }

            // set toml repo
            let toml_repo = TomlRepo {
                local: Some(norm_str.clone()),
                remote,
                branch,
                tag: None,
                commit,
            };
            repos.push(toml_repo);
            logger::new(format!("  + {}", norm_str));

            // just skip go into .git/ folder and continue
            it.skip_current_dir();
            continue;
        }

        count += 1;
    }

    logger::new("");

    // keep list sort same on different device
    repos.sort_by(|a, b| {
        a.local
            .as_ref()
            .unwrap()
            .to_lowercase()
            .cmp(&b.local.as_ref().unwrap().to_lowercase())
    });
    toml_config.repos = Some(repos);
    logger::new(format!("{} files scanned", count));

    // serialize .gitrepos
    let toml_string = toml_config.serialize();
    fs::write(config_path, toml_string).expect("Failed to write file .gitrepos!");
    logger::update_config_succ();
}
