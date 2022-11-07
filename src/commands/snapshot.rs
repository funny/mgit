use super::{display_path, norm_path, SnapshotType, TomlConfig, TomlRepo};
use git2::Repository;
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

    // starting init repos
    println!("init {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if input_path.is_dir() == false {
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

            // try open git repo
            let repo_result = Repository::open(&pb);
            if let Err(e) = repo_result {
                println!(
                    "Failed to open repo {}, {}",
                    display_path(&rel_path.to_str().unwrap().to_string()),
                    e
                );
                continue;
            }
            let repo = repo_result.unwrap();

            // get remote
            let remote = match repo.find_remote("origin") {
                Ok(r) => r.url().map(|s| String::from(s)),
                _ => None,
            };

            let mut commit: Option<String> = None;
            let mut branch: Option<String> = None;

            // set branch or commit-id with '--init' option
            if let Ok(head) = repo.head() {
                if let Some(refname) = head.name() {
                    match snapshot_type {
                        SnapshotType::Commit => {
                            // get local head commit id
                            if let Ok(oid) = repo.refname_to_id(refname) {
                                commit = Some(oid.to_string());
                            }
                        }
                        SnapshotType::Branch => {
                            // get tracking brach
                            if let Ok(buf) = repo.branch_upstream_name(refname) {
                                branch = buf
                                    .as_str()
                                    .map(|str| str.split("refs/remotes/origin/").last())
                                    .unwrap_or(None)
                                    .map(str::to_string)
                            }
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
