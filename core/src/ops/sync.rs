use anyhow::Context;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};
use std::env;
use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::git::{RemoteRef, ResetType, StashMode};
use crate::core::repo::{cmp_local_remote, exclude_ignore};
use crate::core::repo::{RepoId, TomlRepo};
use crate::core::repos::load_config;

use crate::ops::CleanOptions;
use crate::ops::{clean_repo, exec_fetch, set_tracking_remote_branch};

use crate::utils::logger;
use crate::utils::progress::Progress;
use crate::utils::style_message::StyleMessage;

pub struct SyncOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub thread_count: usize,
    pub silent: bool,
    pub depth: Option<usize>,
    pub ignore: Option<Vec<String>>,
    pub hard: bool,
    pub stash: bool,
    pub no_track: bool,
    pub no_checkout: bool,
}

impl SyncOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread_count: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
        hard: Option<bool>,
        stash: Option<bool>,
        no_track: Option<bool>,
        no_checkout: Option<bool>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            thread_count: thread_count.unwrap_or(4),
            silent: silent.unwrap_or(false),
            depth,
            ignore,
            hard: hard.unwrap_or(false),
            stash: stash.unwrap_or(false),
            no_track: no_track.unwrap_or(false),
            no_checkout: no_checkout.unwrap_or(false),
        }
    }
}

pub fn sync_repo(options: SyncOptions, progress: impl Progress) {
    let path = &options.path;
    let config_path = &options.config_path;
    let thread_count = options.thread_count;
    let hard = options.hard;
    let stash = options.stash;
    let silent = options.silent;
    let no_track = options.no_track;
    let no_checkout = options.no_checkout;
    let depth = options.depth.as_ref().copied();
    let ignore = options.ignore.as_ref();

    logger::info(StyleMessage::ops_start("sync repos", path));
    let stash_mode = match (stash, hard) {
        (false, false) => StashMode::Normal,
        (true, false) => StashMode::Stash,
        (false, true) => StashMode::Hard,
        _ => panic!("'--stash' and '--hard' can't be used together."),
    };

    // check if .gitrepos exists
    if !config_path.is_file() {
        logger::error(StyleMessage::config_file_not_found());
        return;
    }

    // load config file(like .gitrepos)
    let Some(toml_config) = load_config(config_path) else{
        logger::error("load config file failed!");
        return;
    };

    // remove unused repositories when use '--config' option
    // also if input_path not exists, skip this process
    if stash_mode == StashMode::Hard && path.is_dir() {
        clean_repo(CleanOptions::new(
            Some(path.clone()),
            Some(config_path.clone()),
        ));
    }

    // load .gitrepos
    let Some(mut toml_repos) = toml_config.repos else {
        return;
    };

    let default_branch = toml_config.default_branch;

    // ignore specified repositories
    let ignore = ignore.map(|r| r.iter().collect::<Vec<&String>>());
    exclude_ignore(&mut toml_repos, ignore);

    let repos_count = toml_repos.len();
    progress.repos_start(repos_count);

    // create thread pool, and set the number of thread to use by using `.num_threads(count)`
    let counter = RelaxedCounter::new(1);
    let thread_builder = rayon::ThreadPoolBuilder::new().num_threads(thread_count);
    let Ok(thread_pool) = thread_builder.build() else
    {
        logger::error("create thread pool failed!");
        return;
    };

    type ParallelResult<'a> = Result<(&'a TomlRepo, StyleMessage), (&'a TomlRepo, anyhow::Error)>;

    // pool.install means that `.par_iter()` will use the thread pool we've built above.
    let (succ_repos, error_repos) = thread_pool.install(|| {
        let res: Vec<ParallelResult> = toml_repos
            .par_iter()
            .map(|toml_repo| {
                let idx = counter.inc();
                let prefix = format!("[{:02}/{:02}]", idx, repos_count);
                let rel_path = toml_repo.local.as_ref().unwrap();

                let msg = format!("{:>9} waiting...", &prefix);
                let progress = progress.clone();
                progress.repo_start(RepoId::new(idx, rel_path));
                progress.repo_info(RepoId::new(idx, rel_path), msg.into());

                // get compare stat betwwen local and specified commit/tag/branch/
                let mut pre_cmp_msg = StyleMessage::new();
                if !silent {
                    let cmp_res = cmp_local_remote(path, toml_repo, &default_branch, false);
                    pre_cmp_msg = pre_cmp_msg.try_join(cmp_res.map_or(None, |m| Some(m)));
                }

                // execute command according each repo status
                let exec_res = inner_exec(
                    path,
                    toml_repo,
                    &stash_mode,
                    no_checkout,
                    depth.as_ref(),
                    &default_branch,
                    &prefix,
                    idx,
                    &progress,
                );

                // handle result
                let rel_path = toml_repo.local.as_ref().unwrap();
                let res: ParallelResult = match exec_res {
                    Ok(_) => {
                        let mut msg = StyleMessage::spinner_end(prefix, rel_path, true);

                        // if not silent, show compare stat betweent local and remote
                        if !silent {
                            let cmp_res = cmp_local_remote(path, toml_repo, &default_branch, false);
                            let mut cmp_msg =
                                StyleMessage::new().try_join(cmp_res.map_or(None, |m| Some(m)));
                            let already_update = cmp_msg.contains("already update to date.");

                            if pre_cmp_msg != cmp_msg && already_update {
                                cmp_msg = cmp_msg.remove("already update to date.");
                                cmp_msg = StyleMessage::git_update_to(cmp_msg);
                            }
                            msg = msg.plain_text(": ").join(cmp_msg)
                        }

                        // show message in progress bar
                        progress.repo_end(RepoId::new(idx, rel_path.as_str()), msg);

                        // track remote branch, return track status
                        let mut track_msg = StyleMessage::new();
                        if !no_track {
                            let track_res =
                                set_tracking_remote_branch(path, &toml_repo, &default_branch);
                            track_msg = track_msg.try_join(track_res.map_or(None, |m| Some(m)));
                        }

                        Ok((toml_repo, track_msg))
                    }
                    Err(e) => {
                        let msg = StyleMessage::spinner_end(
                            prefix,
                            toml_repo.local.as_ref().unwrap(),
                            false,
                        );

                        // show message in progress bar
                        progress.repo_error(RepoId::new(idx, rel_path), msg);

                        Err((toml_repo, e))
                    }
                };

                res
            })
            .collect();

        progress.repos_end();

        // collect repos
        let mut succ_repos: Vec<(&TomlRepo, StyleMessage)> = Vec::new();
        let mut error_repos: Vec<(&TomlRepo, anyhow::Error)> = Vec::new();
        for r in res {
            match r {
                Ok((toml_repo, track_msg)) => succ_repos.push((toml_repo, track_msg)),
                Err((toml_repo, e)) => error_repos.push((toml_repo, e)),
            }
        }
        (succ_repos, error_repos)
    });

    match StyleMessage::ops_errors("sync", error_repos.len()) {
        Ok(msg) => logger::info(msg),
        Err(err_msg) => logger::error(err_msg),
    }

    // show track status
    if !silent {
        logger::info("Track status:");
        succ_repos
            .iter()
            .for_each(|(_, msg)| logger::info(format!("  {}", msg)))
    }

    // show errors
    if !error_repos.is_empty() {
        logger::error("Errors:");
        error_repos.iter().for_each(|(toml_repo, error)| {
            logger::error(StyleMessage::git_error(
                &toml_repo.local.as_ref().unwrap(),
                error,
            ));
        });
    }
}

fn inner_exec(
    input_path: &Path,
    toml_repo: &TomlRepo,
    stash_mode: &StashMode,
    no_checkout: bool,
    depth: Option<&usize>,
    default_branch: &Option<String>,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = &input_path.join(rel_path);

    // make repo directory and skip clone the repository
    std::fs::create_dir_all(full_path)
        .with_context(|| format!("create dir {} failed.", full_path.to_str().unwrap()))?;

    let mut toml_repo = toml_repo.to_owned();
    let mut stash_mode = stash_mode.to_owned();
    let is_repo_none = git::is_repository(full_path.as_path()).is_err();
    // if repository not found, create new one
    if is_repo_none {
        // use --hard
        stash_mode = StashMode::Hard;

        // git init when dir exist
        exec_init(input_path, &toml_repo, prefix, idx, progress)?;
        // git remote add url
        exec_add_remote(input_path, &toml_repo, prefix, idx, progress)?;
    } else {
        let remote_url = toml_repo.remote.as_ref().unwrap();
        git::update_remote_url(full_path, remote_url)?;
    }

    // use default branch when branch is null
    if toml_repo.branch.is_none() {
        toml_repo.branch = default_branch.to_owned();
    }

    // fetch
    exec_fetch(input_path, &toml_repo, depth, prefix, idx, progress)?;

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    // check remote-ref valid
    git::is_remote_ref_valid(full_path, &remote_ref_str)?;

    match stash_mode {
        StashMode::Normal => {
            // try stash → checkout → reset → stash pop
            if !no_checkout {
                // stash
                let stash_result = exec_stash(input_path, &toml_repo, prefix, idx, progress);
                let stash_msg = stash_result.unwrap_or("stash failed.".to_string());

                // checkout
                let mut result: Result<(), anyhow::Error>;
                result = exec_checkout(input_path, &toml_repo, false, prefix, idx, progress);

                if result.is_ok() {
                    // reset --hard
                    result = exec_reset(
                        input_path,
                        &toml_repo,
                        ResetType::Hard,
                        prefix,
                        idx,
                        progress,
                    );
                }

                // stash pop, whether checkout succ or failed, whether reset succ or failed
                if stash_msg.contains("WIP") {
                    let _ = exec_stash_pop(input_path, &toml_repo, prefix, idx, progress);
                }
                result
            } else {
                // reset --soft
                exec_reset(
                    input_path,
                    &toml_repo,
                    ResetType::Soft,
                    prefix,
                    idx,
                    progress,
                )
            }
        }
        StashMode::Stash => {
            // stash with `--stash` option, maybe return error if need to initial commit
            let stash_result = exec_stash(input_path, &toml_repo, prefix, idx, progress);
            let stash_msg = stash_result.unwrap_or("stash failed.".to_string());

            // checkout
            let mut result: Result<(), anyhow::Error> = Ok(());
            let mut reset_type = ResetType::Mixed;
            if !no_checkout {
                result = exec_checkout(input_path, &toml_repo, true, prefix, idx, progress)
                    .with_context(|| stash_msg.clone());

                reset_type = ResetType::Hard;
            }

            // reset --mixed
            if result.is_ok() {
                result = exec_reset(input_path, &toml_repo, reset_type, prefix, idx, progress)
                    .with_context(|| stash_msg.clone());
            }

            // undo if checkout failed or reset failed
            if let Err(e) = result {
                // if reset failed, pop stash if stash something this time
                if stash_msg.contains("WIP") {
                    let _ = exec_stash_pop(input_path, &toml_repo, prefix, idx, progress);
                }
                return Err(e);
            }
            result
        }
        StashMode::Hard => {
            // clean
            if !is_repo_none {
                exec_clean(input_path, &toml_repo, prefix, idx, progress)?;
            }

            // checkout
            if !no_checkout {
                exec_checkout(input_path, &toml_repo, true, prefix, idx, progress)?;
            }

            // reset --hard
            exec_reset(
                input_path,
                &toml_repo,
                ResetType::Hard,
                prefix,
                idx,
                progress,
            )
        }
    }
}

fn exec_init(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "initialize...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    git::init(full_path)
}

fn exec_add_remote(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "add remote...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    let url = toml_repo.remote.as_ref().unwrap();
    git::add_remote_url(full_path, url)
}

fn exec_clean(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "clean...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    git::clean(full_path)
}

fn exec_reset(
    input_path: &Path,
    toml_repo: &TomlRepo,
    reset_type: ResetType,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "reset...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    let reset_type = match reset_type {
        ResetType::Soft => "--soft",
        ResetType::Mixed => "--mixed",
        ResetType::Hard => "--hard",
    };
    git::reset(full_path, reset_type, remote_ref_str)
}

fn exec_stash(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "stash...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    git::stash(full_path)
}

fn exec_stash_pop(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "pop stash...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    git::stash_pop(full_path)
}

fn exec_checkout(
    input_path: &Path,
    toml_repo: &TomlRepo,
    force: bool,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let msg = StyleMessage::spinner_info(prefix, rel_path, "checkout...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };
    let branch = match remote_ref {
        RemoteRef::Commit(commit) => format!("commits/{}", &commit[..7]),
        RemoteRef::Tag(tag) => format!("tags/{}", tag),
        RemoteRef::Branch(_) => toml_repo
            .branch
            .clone()
            .unwrap_or("invalid-branch".to_string()),
    };

    // don't need to checkout if current branch is the branch
    if let Ok(currnte_branch) = git::get_current_branch(full_path.as_path()) {
        if branch == currnte_branch {
            return Ok(());
        }
    }

    let suffix = StyleMessage::git_checking_out(&branch);
    let msg = StyleMessage::spinner_info(prefix, rel_path, suffix);
    progress.repo_info(RepoId::new(idx, rel_path.as_str()), msg);

    // check if local branch already exists
    let branch_exist = git::local_branch_already_exist(&full_path, &branch)?;

    // create/checkout/reset branch
    let args = match (branch_exist, force) {
        (false, false) => vec!["checkout", "-B", &branch, &remote_ref_str, "--no-track"],
        (false, true) => vec![
            "checkout",
            "-B",
            &branch,
            &remote_ref_str,
            "--no-track",
            "-f",
        ],
        (true, false) => vec!["checkout", &branch],
        (true, true) => vec!["checkout", "-B", &branch, "-f"],
    };

    git::checkout(full_path, &args)
}
