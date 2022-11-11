use super::{display_path, load_config};
use globset::GlobBuilder;
use owo_colors::OwoColorize;
use std::{
    env,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn exec(path: Option<String>, config: Option<PathBuf>) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();
    let input_path = Path::new(&input);

    // if directory doesn't exist, finsh clean
    if !input_path.is_dir() {
        println!("Directory {} not found!", input.bold().magenta());
        return;
    }

    // starting clean repos
    println!("Clean Status:");

    // set config file path
    let config_file = match config {
        Some(r) => r,
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if !config_file.is_file() {
        println!(
            "{} not found, try {} instead!",
            ".gitrepos".bold().magenta(),
            "init".bold().magenta()
        );
        return;
    }

    // load .gitrepos
    if let Some(toml_config) = load_config(&config_file) {
        // handle sync
        if let Some(toml_repos) = toml_config.repos {
            let config_repo_paths: Vec<PathBuf> = toml_repos
                .into_iter()
                .map(|item| item.local.unwrap())
                .map(|str| PathBuf::from(&str))
                .collect();

            // search for git repos and create .gitrepos file
            let glob = GlobBuilder::new("**/.git")
                .literal_separator(true)
                .build()
                .unwrap()
                .compile_matcher();

            let mut it = WalkDir::new(input_path).into_iter();
            let mut unused_paths: Vec<PathBuf> = Vec::new();
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
                    if let Err(e) = remove_unused_files(
                        &input_path.to_path_buf(),
                        &unused_path,
                        &contained_paths,
                    ) {
                        println!(
                            "remove {} files error: {}",
                            display_path(&unused_path.to_str().unwrap().to_string()),
                            e
                        )
                    };
                } else {
                    let _ = std::fs::remove_dir_all(input_path.join(&unused_path));
                }
                count += 1;
                println!(
                    "  {}: removed ",
                    display_path(&unused_path.to_str().unwrap().to_string())
                        .bold()
                        .magenta()
                );
            }

            // show statistics info
            if count == 0 {
                println!("no repository is removed.\n");
            } else if count == 1 {
                println!(
                    "{} repository is removed.\n",
                    count.to_string().bold().green()
                );
            } else {
                println!(
                    "{} repositories are removed.\n",
                    count.to_string().bold().green()
                );
            }
        }
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
    base_path: &PathBuf,
    unused_path: &PathBuf,
    contained_paths: &Vec<PathBuf>,
) -> Result<(), anyhow::Error> {
    let full_path = base_path.join(&unused_path);

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
