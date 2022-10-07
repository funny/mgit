use std::path::{Path};
use std::collections::{HashMap};
use std::fs;
use walkdir::WalkDir;
use globset::GlobBuilder;
use toml_edit::easy as toml;
use super::{TomlConfig, TomlRepo};

pub fn exec(path: Option<String>) {
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

    // TODO
    // // check if .mgit/ exists
    // let user_dir = input_path.join(".mgit");
    // if user_dir.is_dir() == false {
    // }

    // check if .gitrepos exists
    let config_file = input_path.join(".gitrepos");
    if config_file.is_file() == true {
        println!("The directory {} already inited!", input);
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
            let rel_path = Path::new(".").join(
                pb.strip_prefix(input_path) .unwrap()
            );
            let norm_path = rel_path
                .into_os_string()
                .into_string()
                .unwrap()
                .replace("\\", "/")
                ;

            // DELME:
            // convert project name
            // let name = match rel_path.clone().file_name() {
            //     None => String::from("_"),
            //     Some(p) => p.to_os_string().into_string().unwrap(),
            // };
            // let name2 = name.replace(".", "_");
            // println!("project name = {}", name2);

            // set toml repo
            let toml_repo = TomlRepo {
                local: Some(norm_path.clone()),
                remote: None,
                branch: None,
                tag: None,
                commit: None,
            };
            repos.push(toml_repo);
            println!("add {}", norm_path);

            // just skip go into .git/ folder and continue
            it.skip_current_dir();
            continue;
        }

        count += 1;
    }
    let mut repos_map = HashMap::new();
    repos_map.insert(String::from("repos"), repos);
    toml_config.repos = Some(repos_map);
    // toml_config.repos = Some(repos);
    println!("{} files read!", count);

    let toml_string = toml::to_string(&toml_config).unwrap();
    fs::write(config_file, toml_string).expect("Failed to write file .gitrepos!");
}
