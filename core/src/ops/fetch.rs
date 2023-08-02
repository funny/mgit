use atomic_counter::{AtomicCounter, RelaxedCounter};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::git;
use crate::core::git::RemoteRef;
use crate::core::repo::TomlRepo;
use crate::core::repo::{cmp_local_remote, exclude_ignore};
use crate::core::repos::load_config;

use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::style_message::StyleMessage;
use crate::utils::{cmd, logger};

pub struct FetchOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub thread_count: usize,
    pub silent: bool,
    pub depth: Option<usize>,
    pub ignore: Option<Vec<String>>,
}

impl FetchOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            thread_count: thread.unwrap_or(4),
            silent: silent.unwrap_or(false),
            depth,
            ignore,
        }
    }
}

pub fn fetch_repos(options: FetchOptions, progress: impl Progress) {
    let path = &options.path;
    let config_path = &options.config_path;
    let thread_count = options.thread_count;
    let silent = options.silent;
    let depth = options.depth.as_ref().copied();
    let ignore = options.ignore.as_ref();

    // start fetching repos
    logger::info(StyleMessage::ops_start("fetch repos", path));

    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::info(StyleMessage::config_file_not_found());
        return;
    }
    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(config_path) else {
        logger::error("load config file failed!");
        return;
    };

    let Some(toml_repos) = toml_config.repos else {
        return;
    };
    let default_branch = toml_config.default_branch;

    // ignore specified repositories
    let mut toml_repos = toml_repos
        .into_iter()
        .enumerate()
        .collect::<HashMap<usize, TomlRepo>>();
    exclude_ignore(
        &mut toml_repos,
        ignore.map(|it| it.iter().collect::<Vec<&String>>()),
    );

    progress.repos_start(toml_repos.len());

    // user a counter
    let counter = RelaxedCounter::new(1);

    // create thread pool, and set the number of thread to use by using `.num_threads(count)`
    let thread_builder = rayon::ThreadPoolBuilder::new().num_threads(thread_count);
    let Ok(thread_pool) = thread_builder.build() else {
        logger::error("create thread pool failed!");
        return;
    };

    // pool.install means that `.par_iter()` will use the thread pool we've built above.
    let errors: Vec<(&TomlRepo, anyhow::Error)> = thread_pool.install(|| {
        let res = toml_repos
            .iter()
            // .enumerate()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(index, toml_repo)| {
                let idx = counter.inc();
                let repo_info = RepoInfo::new(*index, idx, toml_repo);

                let progress = progress.clone();
                progress.repo_start(&repo_info, "waiting...".into());

                // execute fetch command with progress
                let exec_res = inner_exec(path, &repo_info, depth.as_ref(), &progress);

                // handle result
                match exec_res {
                    Ok(_) => {
                        // if not silent, show compare stat between local and remote
                        let msg = match silent {
                            true => StyleMessage::new(),
                            false => StyleMessage::from(" ").try_join(
                                cmp_local_remote(path, toml_repo, &default_branch, false).ok(),
                            ),
                        };
                        progress.repo_end(&repo_info, msg);
                        Ok(())
                    }
                    Err(e) => {
                        progress.repo_error(&repo_info, StyleMessage::new());
                        Err((toml_repo, e))
                    }
                }
            })
            .filter_map(Result::err)
            .collect();

        progress.repos_end();
        res
    });

    match StyleMessage::ops_errors("fetch", errors.len()) {
        Ok(msg) => logger::info(msg),
        Err(err_msg) => logger::error(err_msg),
    }

    // show errors
    if !errors.is_empty() {
        logger::error("Errors:");
        errors.iter().for_each(|(toml_repo, error)| {
            logger::error(StyleMessage::git_error(
                toml_repo.local.as_ref().unwrap(),
                error,
            ));
        });
    }
}

fn inner_exec(
    input_path: impl AsRef<Path>,
    repo_info: &RepoInfo,
    depth: Option<&usize>,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let full_path = input_path.as_ref().join(repo_info.rel_path());
    let remote_url = repo_info.toml_repo.remote.as_ref().unwrap();

    git::update_remote_url(full_path, remote_url)?;
    exec_fetch(input_path, repo_info, depth, progress)
}

pub fn exec_fetch(
    input_path: impl AsRef<Path>,
    repo_info: &RepoInfo,
    depth: Option<&usize>,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let full_path = input_path.as_ref().join(repo_info.rel_path());

    // get remote name from url
    let remote_name = repo_info.toml_repo.get_remote_name(full_path.as_path())?;
    let mut args = vec!["fetch", &remote_name];

    if let Some(depth) = depth {
        // priority: commit/tag/branch(default-branch)
        let remote_ref = repo_info.toml_repo.get_remote_ref(full_path.as_path())?;
        match remote_ref {
            RemoteRef::Commit(commit) => {
                args.push(Box::leak(commit.into_boxed_str()));
            }
            RemoteRef::Tag(tag) => {
                args.push("tag");
                args.push(Box::leak(tag.into_boxed_str()));
                args.push("--no-tags");
            }
            RemoteRef::Branch(_) => {
                let branch = repo_info.toml_repo.branch.as_ref().expect("invalid-branch");
                args.push(branch);
            }
        };

        args.push("--depth");
        args.push(Box::leak(depth.to_string().into_boxed_str()));
    }

    args.push("--prune");
    args.push("--recurse-submodules=on-demand");
    args.push("--progress");

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(full_path);

    cmd::exec_cmd_with_progress(repo_info, full_command, progress)
}
