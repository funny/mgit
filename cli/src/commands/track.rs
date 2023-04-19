use clap::ArgMatches;
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    config::{
        repo::{exclude_ignore, TomlRepo},
        repos::{load_config, TomlConfig},
    },
    git,
    utils::logger,
};

use super::RemoteRef;

pub(crate) fn exec(args: &ArgMatches) {
    // get input path
    let input_path = match args.get_one::<String>("path") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    // starting clean repos
    logger::new("Track status:");

    // if directory doesn't exist, finsh clean
    if !input_path.is_dir() {
        logger::dir_not_found(&input_path);
        return;
    }

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
    if !config_file.is_file() {
        logger::config_file_not_found();
        return;
    }

    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(&config_file) else{
        logger::new("load config file failed!");
        return;
    };

    inner_exec(input_path, toml_config, ignore)
}

fn inner_exec(input_path: impl AsRef<Path>, toml_config: TomlConfig, ignore: Option<Vec<&String>>) {
    // handle track
    let Some(mut toml_repos) = toml_config.repos else {
        return;
    };

    let default_branch = toml_config.default_branch;

    // ignore specified repositories
    exclude_ignore(&mut toml_repos, ignore);

    for toml_repo in &toml_repos {
        if let Ok(res) = set_tracking_remote_branch(&input_path, toml_repo, &default_branch) {
            logger::new(format!("  {}", res));
        }
    }
}

pub(crate) fn set_tracking_remote_branch(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    default_branch: &Option<String>,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);

    // get local current branch
    let local_branch = git::get_current_branch(full_path.as_path())?;

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
        let res = logger::fmt_untrack_desc(rel_path, &remote_desc);
        return Ok(res);
    }

    git::set_tracking_remote_branch(
        full_path,
        rel_path,
        local_branch,
        remote_ref_str,
        remote_desc,
    )
}
