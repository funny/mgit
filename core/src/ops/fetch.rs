use atomic_counter::{AtomicCounter, RelaxedCounter};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::git;
use crate::core::git::RemoteRef;
use crate::core::repo::{cmp_local_remote, exclude_ignore};
use crate::core::repo::{RepoId, TomlRepo};
use crate::core::repos::load_config;

use crate::utils::progress::Progress;
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

    let Some(mut toml_repos) = toml_config.repos else {
        return;
    };
    let default_branch = toml_config.default_branch;

    // ignore specified repositories
    exclude_ignore(
        &mut toml_repos,
        ignore.map(|it| it.iter().collect::<Vec<&String>>()),
    );

    let repos_count = toml_repos.len();
    progress.repos_start(repos_count);

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
            .par_iter()
            .map(|toml_repo| {
                let idx = counter.inc();
                let prefix = format!("[{:02}/{:02}]", idx, repos_count);
                let rel_path = toml_repo.local.as_ref().unwrap();

                let msg = StyleMessage::spinner_start(&prefix, " waiting...");
                let progress = progress.clone();
                progress.repo_start(RepoId::new(idx, rel_path));
                progress.repo_info(RepoId::new(idx, rel_path), msg);

                // execute fetch command with progress
                let exec_res = inner_exec(path, toml_repo, depth.as_ref(), &prefix, idx, &progress);

                // handle result
                let res = match exec_res {
                    Ok(_) => {
                        let mut msg = StyleMessage::spinner_end(prefix, rel_path, true);

                        // if not silent, show compare stat betweent local and remote
                        if !silent {
                            let cmp_res = cmp_local_remote(path, toml_repo, &default_branch, false);
                            msg = msg.try_join(cmp_res.map_or(None, |m| Some(m)));
                        }

                        progress.repo_end(RepoId::new(idx, rel_path.as_str()), msg);
                        Ok(())
                    }
                    Err(e) => {
                        let msg = StyleMessage::spinner_end(prefix, rel_path, false);
                        progress.repo_error(RepoId::new(idx, rel_path.as_str()), msg);
                        Err((toml_repo, e))
                    }
                };
                res
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
                &toml_repo.local.as_ref().unwrap(),
                error,
            ));
        });
    }
}

fn inner_exec(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    depth: Option<&usize>,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);
    let remote_url = toml_repo.remote.as_ref().unwrap();

    git::update_remote_url(full_path, remote_url)?;
    exec_fetch(input_path, toml_repo, depth, &prefix, idx, progress)
}

pub fn exec_fetch(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    depth: Option<&usize>,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);

    // get remote name from url
    let remote_name = toml_repo.get_remote_name(full_path.as_path())?;
    let mut args = vec!["fetch", &remote_name];

    if let Some(depth) = depth {
        // priority: commit/tag/branch(default-branch)
        let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
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
                let branch = toml_repo.branch.as_ref().expect("invalid-branch");
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

    cmd::exec_cmd_with_progress(rel_path, full_command, prefix, idx, progress)
}
