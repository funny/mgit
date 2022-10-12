use super::{TomlConfig, TomlRepo};
use git2::Repository;
use globset::GlobBuilder;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn exec(path: Option<String>, force: bool) {
    let cwd = std::env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    // starting init repos
    println!("init {}", input);
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if input_path.is_dir() == false {
        println!("Invalid input: directory {} not found!", input);
        return;
    }

    // check if .gitrepos exists
    let config_file = input_path.join(".gitrepos");
    if config_file.is_file() && !force {
        println!("{} already inited, try --force instead!", input);
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
            let repo_result = Repository::open(pb);
            if let Err(e) = repo_result {
                println!("Failed to open repo {}, {}", rel_path.display(), e);
                continue;
            }
            let repo = repo_result.unwrap();

            // get remote
            let remote = match repo.find_remote("origin") {
                Ok(r) => r.url().map(|s| String::from(s)),
                _ => None,
            };

            // get branch
            let branch = match repo.head() {
                Ok(r) => {
                    if r.is_branch() {
                        r.shorthand().map(|s| String::from(s))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // normalize path if needed
            let norm_path = rel_path
                .into_os_string()
                .into_string()
                .unwrap()
                .replace("\\", "/");

            // if git in root path, represent it by "."
            let norm_str = if norm_path.is_empty() {
                "."
            } else {
                norm_path.as_str()
            };

            // set toml repo
            let toml_repo = TomlRepo {
                local: Some(String::from(norm_str)),
                remote,
                branch,
                tag: None,
                commit: None,
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

    toml_config.repos = Some(repos);
    println!("{} files scanned", count);

    // serialize .gitrepos
    let toml_string = toml_config.serialize();
    fs::write(config_file, toml_string).expect("Failed to write file .gitrepos!");
    println!(".gitrepos update");
}
