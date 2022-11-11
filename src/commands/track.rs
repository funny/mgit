use super::{display_path, execute_cmd, get_current_branch, load_config, RemoteRef, TomlRepo};
use owo_colors::OwoColorize;
use std::{
    env,
    path::{Path, PathBuf},
};

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

    // start set track remote branch
    println!("Track status:");

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
        let default_branch = toml_config.default_branch;

        // handle sync
        if let Some(toml_repos) = toml_config.repos {
            for toml_repo in toml_repos {
                if let Ok(res) = set_tracking_remote_branch(input_path, &toml_repo, &default_branch)
                {
                    println!("  {}", res);
                }
            }
        }
    }
}

pub fn set_tracking_remote_branch(
    input_path: &Path,
    toml_repo: &TomlRepo,
    default_branch: &Option<String>,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    // get local current branch
    let local_branch = get_current_branch(full_path.as_path())?;

    let mut toml_repo = toml_repo.to_owned();
    // use default branch when branch is null
    if None == toml_repo.branch {
        toml_repo.branch = default_branch.to_owned();
    }

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(commit) => commit,
        RemoteRef::Tag(tag) => tag,
        RemoteRef::Branch(branch) => branch,
    };
    let remote_desc = match remote_ref {
        RemoteRef::Commit(commit) => (&commit[..7]).to_string(),
        RemoteRef::Tag(tag) => tag,
        RemoteRef::Branch(branch) => branch,
    };

    if toml_repo.commit.is_some() || toml_repo.tag.is_some() {
        let res = format!(
            "{}: {} {}",
            display_path(rel_path).bold().magenta(),
            remote_desc.blue(),
            "untracked"
        );
        return Ok(res);
    }

    // git branch --set-upstream-to <name>
    // true only when remote head is branch
    let args = vec!["branch", "--set-upstream-to", &remote_ref_str];

    if execute_cmd(&full_path, "git", &args).is_ok() {
        let res = format!(
            "{}: {} -> {}",
            display_path(rel_path).bold().magenta(),
            local_branch.blue(),
            remote_desc.blue()
        );
        Ok(res)
    } else {
        let res = format!(
            "{}: {} {} {}",
            display_path(rel_path).bold().magenta(),
            "track failed,".red(),
            remote_desc.blue(),
            "not found!".red()
        );
        Ok(res)
    }
}
