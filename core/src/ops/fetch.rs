use atomic_counter::{AtomicCounter, RelaxedCounter};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use anyhow::anyhow;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::core::git;
use crate::core::git::RemoteRef;
use crate::core::repo::{cmp_local_remote, repos_to_map_with_ignore};
use crate::core::repos::TomlConfig;

use crate::utils::cmd::retry;
use crate::utils::error::{MgitError, MgitResult, OpsErrors};
use crate::utils::path::PathExtension;
use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::style_message::StyleMessage;
use crate::utils::{cmd, label, logger};

pub struct FetchOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub thread_count: usize,
    pub silent: bool,
    pub depth: Option<usize>,
    pub ignore: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

impl FetchOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
        labels: Option<Vec<String>>,
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
            labels,
        }
    }
}

pub fn fetch_repos(options: FetchOptions, progress: impl Progress) -> MgitResult {
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
        return Err(anyhow!(MgitError::ConfigFileNotFound(
            StyleMessage::config_file_not_found(),
        )));
    }
    // load config file(like .gitrepos)
    let Some(toml_config) = TomlConfig::load(config_path) else {
        return Err(anyhow!(MgitError::LoadConfigFailed));
    };

    let Some(mut toml_repos) = toml_config.repos else {
        return Ok("No repos to fetch".into());
    };

    if let Some(labels) = options.labels {
        toml_repos = label::filter(&toml_repos, &labels).cloned().collect();
    }

    let default_branch = toml_config.default_branch;

    // retain repos exclude ignore repositories
    let repos_map = repos_to_map_with_ignore(toml_repos, ignore);

    progress.repos_start(repos_map.len());

    // use a counter
    let counter = RelaxedCounter::new(1);

    // create thread pool, and set the number of thread to use by using `.num_threads(count)`
    let thread_builder = rayon::ThreadPoolBuilder::new().num_threads(thread_count);
    let Ok(thread_pool) = thread_builder.build() else {
        return Err(anyhow!(MgitError::CreateThreadPoolFailed));
    };

    // pool.install means that `.par_iter()` will use the thread pool we've built above.
    let errors: Vec<_> = thread_pool.install(|| {
        let res = repos_map
            .iter()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(id, toml_repo)| {
                let index = counter.inc();
                let repo_info = RepoInfo::new(*id, index, toml_repo);

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
                            false => cmp_local_remote(path, toml_repo, &default_branch, false)
                                .unwrap_or(StyleMessage::new()),
                        };
                        progress.repo_end(&repo_info, msg);
                        Ok(())
                    }
                    Err(e) => {
                        progress.repo_error(&repo_info, StyleMessage::new());
                        Err(StyleMessage::git_error(
                            toml_repo.local.as_ref().unwrap().display_path(),
                            &e,
                        ))
                    }
                }
            })
            .filter_map(Result::err)
            .collect();

        progress.repos_end();
        res
    });

    match errors.len() {
        0 => Ok(StyleMessage::ops_success("fetch")),
        _ => {
            let msg = StyleMessage::ops_failed("fetch", errors.len());
            Err(anyhow!(MgitError::OpsError {
                prefix: msg,
                errors: OpsErrors(errors),
            }))
        }
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

    retry(10, Duration::from_millis(400), || {
        let args = args.clone();
        let full_path = full_path.clone();
        let mut command = Command::new("git");
        let full_command = command.args(args).current_dir(full_path);
        cmd::exec_cmd_with_progress(repo_info, full_command, progress)
    })
}
