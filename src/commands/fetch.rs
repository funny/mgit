use super::load_config;
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
        println!("{:?}", toml_config);
    }
}
