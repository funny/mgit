use super::load_config;
use git2::Repository;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn exec(path: Option<String>) {
    let cwd = std::env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    // starting fetch repos
    println!("fetch {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if input_path.is_dir() == false {
        println!("Directory {} not found!", input.bold().magenta());
        return;
    }

    // check if .gitrepos exists
    let config_file = input_path.join(".gitrepos");
    if config_file.is_file() == false {
        println!(
            "{} not found, try {} instead!",
            ".gitrepos".bold().magenta(),
            "init".bold().magenta()
        );
        return;
    }

    // load .gitrepos
    if let Some(toml_config) = load_config(input_path) {
        // println!("{:?}", toml_config);
        let input_path_buf = input_path.to_path_buf();

        if let Some(repos) = toml_config.repos {
            for toml_repo in &repos {
                println!("");

                // open local repository
                let repo = toml_repo.local.as_ref().and_then(|local| {
                    println!("open repo: {}", local.bold().magenta());
                    let repo_path = input_path_buf.join(local);
                    let repo_result = Repository::open(&repo_path);

                    if let Err(e) = &repo_result {
                        println!(
                            "Failed to open repo {}, {}",
                            repo_path.display().bold().magenta(),
                            e
                        );
                    }

                    repo_result.ok()
                });

                // find remote
                if let Some(repo) = repo {
                    // try to fetch remote by url
                    let remote = toml_repo.remote.as_ref().and_then(|remote_url| {
                        println!("connect remote: {}", remote_url.bold().magenta());
                        let remote_result = repo.remote_anonymous(&remote_url);

                        match remote_result {
                            Err(_) => {
                                let url = toml_config
                                    .default_remote
                                    .as_ref()
                                    .map(|def_remote| format!("{}/{}", def_remote, remote_url))
                                    .unwrap();
                                let remote_result2 = repo.remote_anonymous(&url);

                                if let Err(e2) = &remote_result2 {
                                    println!(
                                        "Failed to fetch remote {}, {}",
                                        remote_url.bold().magenta(),
                                        e2
                                    );
                                }

                                remote_result2.ok()
                            }

                            Ok(r) => Some(r),
                        }
                    });

                    // fetch remote
                    if let Some(mut remote) = remote {
                        if let Err(e) = remote.fetch(&["master"], None, None) {
                            println!("Failed to fetch branch: {}, {}", "main", e);
                        } else {
                            println!("{} fetched!", remote.url().unwrap());
                        }
                    }
                }
            }
        }
    }
}
