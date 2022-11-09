use super::{
    display_path, find_remote_url_by_name, get_current_commit, get_tracking_branch, is_repository,
    norm_path, SnapshotType, TomlConfig, TomlRepo,
};
use globset::GlobBuilder;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn exec(
    path: Option<String>,
    config: Option<PathBuf>,
    snapshot_type: SnapshotType,
    force: bool,
) {
    let cwd = std::env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    //  start taking snapshot repos
    println!("take snapshot in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if !input_path.is_dir() {
        println!("Directory {} not found!", input.bold().magenta());
        return;
    }

    // set config file path
    let config_file = match config {
        Some(r) => r,
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if config_file.is_file() && !force {
        println!(
            "{} already inited, try {} instead!",
            input,
            "--force".bold().magenta()
        );
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

    println!("search and add git repos:");
    let mut count = 0;
    let mut it = WalkDir::new(input_path).into_iter();
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
            let rel_path = pb.strip_prefix(input_path).unwrap().to_path_buf();

            // check repository valid
            if is_repository(pb.as_path()).is_err() {
                println!(
                    "Failed to open repo {}!",
                    display_path(&rel_path.to_str().unwrap().to_string()),
                );
                continue;
            }

            // get remote
            let remote = match find_remote_url_by_name(pb.as_path(), &"origin".to_string()) {
                Ok(r) => Some(r),
                _ => None,
            };

            let mut commit: Option<String> = None;
            let mut branch: Option<String> = None;

            // snapshot commit or remote-branch
            match snapshot_type {
                SnapshotType::Commit => {
                    // get local head commit id
                    if let Ok(oid) = get_current_commit(pb.as_path()) {
                        commit = Some(oid);
                    }
                }
                SnapshotType::Branch => {
                    // get tracking brach
                    if let Ok(refname) = get_tracking_branch(pb.as_path()) {
                        // split, like origin/master
                        if let Some((_, branch_ref)) = refname.split_once("/") {
                            branch = Some(branch_ref.trim().to_string());
                        }
                    }
                }
            }

            // normalize path if needed
            let norm_path = norm_path(&rel_path.to_str().unwrap().to_string());

            // if git in root path, represent it by "."
            let norm_str = &display_path(&norm_path);

            // set toml repo
            let toml_repo = TomlRepo {
                local: Some(String::from(norm_str)),
                remote,
                branch,
                tag: None,
                commit,
            };
            repos.push(toml_repo);
            println!("  + {}", norm_str);

            // just skip go into .git/ folder and continue
            it.skip_current_dir();
            continue;
        }

        count += 1;
    }

    println!("");

    // keep list sort same on different device
    repos.sort_by(|a, b| {
        a.local
            .as_ref()
            .unwrap()
            .to_lowercase()
            .cmp(&b.local.as_ref().unwrap().to_lowercase())
    });
    toml_config.repos = Some(repos);
    println!("{} files scanned", count);

    // serialize .gitrepos
    let toml_string = toml_config.serialize();
    fs::write(config_file, toml_string).expect("Failed to write file .gitrepos!");
    println!("{} update", ".gitrepos".bold().magenta());
}
