use clap::ArgMatches;
use globset::GlobBuilder;
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

use super::SnapshotType;
use crate::config::repo::TomlRepo;
use crate::config::repos::TomlConfig;
use crate::git;
use crate::utils::logger;
use crate::utils::path::{display_path, norm_path};

pub(crate) fn exec(args: &ArgMatches) {
    // get input path
    let input_path = match args.get_one::<String>("path") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    // start taking snapshot repos
    logger::command_start("take snapshot", &input_path);

    // if directory doesn't exist, finsh clean
    if !input_path.is_dir() {
        logger::dir_not_found(&input_path);
        return;
    }

    // get snapshot type
    let snapshot_type = match args.get_one::<bool>("branch").unwrap_or(&false) {
        true => SnapshotType::Branch,
        false => SnapshotType::Commit,
    };

    // get force flag
    let force = args.get_one::<bool>("force").unwrap_or(&false);

    // get ignore
    let ignore = match args.get_many::<String>("ignore") {
        Some(r) => {
            let ignore = r.collect::<Vec<&String>>();
            Some(ignore)
        }
        _ => None,
    };

    // set config file path
    let config_file = match args.get_one::<PathBuf>("config") {
        Some(r) => r.to_owned(),
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if config_file.is_file() && !force {
        logger::dir_already_inited(&input_path);
        return;
    }

    inner_exec(input_path, &config_file, snapshot_type, ignore);
}

fn inner_exec(
    input_path: impl AsRef<Path>,
    config_file: impl AsRef<Path>,
    snapshot_type: SnapshotType,
    ignore: Option<Vec<&String>>,
) {
    exec_snapshot(input_path, config_file, snapshot_type, ignore);
}

pub(crate) fn exec_snapshot(
    input_path: impl AsRef<Path>,
    config_file: impl AsRef<Path>,
    snapshot_type: SnapshotType,
    ignore: Option<Vec<&String>>,
) {
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
    fs::write(config_file, toml_string).expect("Failed to write file .gitrepos!");
    logger::update_config_succ();
}
