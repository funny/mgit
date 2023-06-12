use atomic_counter::{AtomicCounter, RelaxedCounter};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::core::git;
use crate::core::git::RemoteRef;
use crate::core::repo::TomlRepo;
use crate::core::repo::{cmp_local_remote, exclude_ignore};
use crate::core::repos::load_config;

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

pub fn fetch_repos(options: FetchOptions) {
    let path = &options.path;
    let config_path = &options.config_path;
    let thread_count = options.thread_count;
    let silent = options.silent;
    let depth = options.depth.as_ref().copied();
    let ignore = options.ignore.as_ref();

    // start fetching repos
    logger::command_start("fetch repos", path);
    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::config_file_not_found();
        return;
    }
    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(config_path) else{
        logger::new("load config file failed!");
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

    // multi_progress manages multiple progress bars from different threads
    // use Arc to share the MultiProgress across more than 1 thread
    let multi_progress = Arc::new(MultiProgress::new());
    // create total progress bar and set progress style
    let total_bar = multi_progress.add(ProgressBar::new(repos_count as u64));
    total_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {percent}% [{bar:30.green/white}] {pos}/{len}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    total_bar.enable_steady_tick(std::time::Duration::from_millis(500));

    // user a counter
    let counter = RelaxedCounter::new(1);

    // Clone Arc<MultiProgress> and spawn a thread.
    // need to do this in a thread as the `.map()` we do below also blocks.
    let multi_progress_wait = multi_progress.clone();

    // create thread pool, and set the number of thread to use by using `.num_threads(count)`
    let thread_builder = rayon::ThreadPoolBuilder::new().num_threads(thread_count);
    let Ok(thread_pool) = thread_builder.build() else
    {
        logger::new("create thread pool failed!");
        return;
    };

    // pool.install means that `.par_iter()` will use the thread pool we've built above.
    let errors: Vec<(&TomlRepo, anyhow::Error)> = thread_pool.install(|| {
        let res = toml_repos
            .par_iter()
            .map(|toml_repo| {
                let idx = counter.inc();
                let prefix = format!("[{:02}/{:02}]", idx, repos_count);

                // create progress bar for each repo
                let progress_bar = multi_progress_wait.insert(idx, ProgressBar::new_spinner());
                progress_bar.set_style(
                    ProgressStyle::with_template("{spinner:.green.dim.bold} {msg} ")
                        .unwrap()
                        .tick_chars("/-\\| "),
                );
                progress_bar.enable_steady_tick(std::time::Duration::from_millis(500));
                let msg = logger::fmt_spinner_start(&prefix, " waiting...");
                progress_bar.set_message(logger::truncate_spinner_msg(&msg));

                // execute fetch command with progress
                let exec_result = inner_exec_with_progress(
                    path,
                    toml_repo,
                    depth.as_ref(),
                    &prefix,
                    &progress_bar,
                );

                // handle result
                let rel_path = toml_repo.local.as_ref().unwrap();
                let result = match exec_result {
                    Ok(_) => {
                        let mut msg = logger::fmt_spinner_finished_prefix(prefix, rel_path, true);

                        // if not silent, show compare stat betweent local and remote
                        if !silent {
                            match cmp_local_remote(path, toml_repo, &default_branch, false) {
                                Ok(r) => msg = format!("{}: {}", msg, r.unwrap()),
                                _ => {}
                            };
                        };

                        progress_bar.finish_with_message(logger::truncate_spinner_msg(&msg));
                        Ok(())
                    }
                    Err(e) => {
                        let msg = logger::fmt_spinner_finished_prefix(prefix, rel_path, false);

                        progress_bar.finish_with_message(logger::truncate_spinner_msg(&msg));
                        Err((toml_repo, e))
                    }
                };

                // update total progress bar
                total_bar.inc(1);

                result
            })
            .filter_map(Result::err)
            .collect();

        total_bar.finish();

        res
    });

    logger::new("\n");
    logger::error_statistics("fetch", errors.len());

    // show errors
    if !errors.is_empty() {
        logger::new("Errors:");
        errors.iter().for_each(|(toml_repo, error)| {
            logger::error_detail(&toml_repo.local.as_ref().unwrap(), error);
        });
    }
}

fn inner_exec_with_progress(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    depth: Option<&usize>,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.as_ref().join(rel_path);
    let remote_url = toml_repo.remote.as_ref().unwrap();

    git::update_remote_url(full_path, remote_url)?;

    exec_fetch_with_progress(input_path, toml_repo, depth, &prefix, &progress_bar)
}

pub fn exec_fetch_with_progress(
    input_path: impl AsRef<Path>,
    toml_repo: &TomlRepo,
    depth: Option<&usize>,
    prefix: &str,
    progress_bar: &ProgressBar,
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

    cmd::exec_cmd_with_progress(rel_path, full_command, prefix, progress_bar)
}
