use anyhow::{anyhow, Context};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::env;
use std::path::{Path, PathBuf};

use crate::core::git;
use crate::core::git::{RemoteRef, ResetType, StashMode};
use crate::core::repo::{cmp_local_remote, repos_to_map_with_ignore};
use crate::core::repos::TomlConfig;

use crate::ops::CleanOptions;
use crate::ops::{clean_repo, exec_fetch, set_tracking_remote_branch};
use crate::utils::error::{MgitError, MgitResult, OpsErrors};

use crate::utils::logger;
use crate::utils::path::PathExtension;
use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Default)]
struct InnerExecResponse {
    stash: Option<InnerStashResponse>,
}

#[derive(Debug)]
enum InnerStashResponse {
    None,
    Stash(String),
}

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
    #[allow(clippy::too_many_arguments)]
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

pub fn sync_repo(options: SyncOptions, progress: impl Progress) -> MgitResult {
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
        return Err(anyhow!(MgitError::ConfigFileNotFound(
            StyleMessage::config_file_not_found(),
        )));
    }

    // load config file(like .gitrepos)
    let Some(toml_config) = TomlConfig::load(config_path) else {
        return Err(anyhow!(MgitError::LoadConfigFailed));
    };

    // remove unused repositories when use '--config' option
    // also if input_path not exists, skip this process
    if stash_mode == StashMode::Hard && path.is_dir() {
        clean_repo(CleanOptions::new(
            Some(path.clone()),
            Some(config_path.clone()),
        ))?;
    }

    // load .gitrepos
    let Some(toml_repos) = toml_config.repos else {
        return Ok("No repos to sync".into());
    };

    let default_branch = toml_config.default_branch;

    // retain repos exclude ignore repositories
    let repos_map = repos_to_map_with_ignore(toml_repos, ignore);

    progress.repos_start(repos_map.len());

    // create thread pool, and set the number of thread to use by using `.num_threads(count)`
    let counter = RelaxedCounter::new(1);
    let thread_builder = rayon::ThreadPoolBuilder::new().num_threads(thread_count);
    let Ok(thread_pool) = thread_builder.build() else {
        return Err(anyhow!(MgitError::CreateThreadPoolFailed));
    };

    struct SuccRepoInfo {
        stash_status: StyleMessage,
        track_status: StyleMessage,
    }

    type ParallelResult<'a> = Result<SuccRepoInfo, StyleMessage>;

    // pool.install means that `.par_iter()` will use the thread pool we've built above.
    let (succ_repos, error_repos) = thread_pool.install(|| {
        let res: Vec<ParallelResult> = repos_map
            .iter()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(id, toml_repo)| {
                let index = counter.inc();
                let mut repo_info = RepoInfo::new(*id, index, toml_repo);

                let progress = progress.clone();
                progress.repo_start(&repo_info, "waiting...".into());

                // get compare stat betwwen local and specified commit/tag/branch/
                let mut pre_cmp_msg = StyleMessage::new();
                if !silent {
                    let cmp_res = cmp_local_remote(path, toml_repo, &default_branch, false);
                    pre_cmp_msg = pre_cmp_msg.try_join(cmp_res.ok());
                }

                // execute command according each repo status
                let exec_res = inner_exec(
                    path,
                    &mut repo_info,
                    &stash_mode,
                    no_checkout,
                    depth.as_ref(),
                    &default_branch,
                    &progress,
                );

                // handle result
                match exec_res {
                    Ok(response) => {
                        // if not silent, show compare stat betweent local and remote
                        let msg = match silent {
                            true => StyleMessage::new(),
                            false => {
                                let mut cmp_msg =
                                    cmp_local_remote(path, toml_repo, &default_branch, false)
                                        .unwrap_or(StyleMessage::new());
                                let already_update = cmp_msg.contains("already update to date.");

                                if pre_cmp_msg != cmp_msg && already_update {
                                    cmp_msg = cmp_msg.remove("already update to date.");
                                    cmp_msg = StyleMessage::git_update_to(cmp_msg);
                                }
                                cmp_msg
                            }
                        };

                        // show message in progress bar
                        progress.repo_end(&repo_info, msg);

                        // stash status: stash on some commit
                        let mut stash_status = StyleMessage::new();
                        if let Some(InnerStashResponse::Stash(msg)) = response.stash {
                            let repo_rel_path = toml_repo.local.as_ref().unwrap().display_path();
                            stash_status =
                                stash_status.join(StyleMessage::git_stash(repo_rel_path, msg));
                        }

                        // track status: track remote branch
                        let mut track_status = StyleMessage::new();
                        if !no_track {
                            let track_res =
                                set_tracking_remote_branch(path, toml_repo, &default_branch);
                            track_status = track_status.try_join(track_res.ok());
                        }

                        let info = SuccRepoInfo {
                            stash_status,
                            track_status,
                        };
                        Ok(info)
                    }
                    Err(e) => {
                        // show message in progress bar
                        progress.repo_error(&repo_info, StyleMessage::new());

                        let repo_rel_path = toml_repo.local.as_ref().unwrap().display_path();
                        Err(StyleMessage::git_error(repo_rel_path, &e))
                    }
                }
            })
            .collect();

        progress.repos_end();

        // collect repos
        let mut succ_repos = Vec::new();
        let mut error_repos = Vec::new();
        for r in res {
            match r {
                Ok(info) => succ_repos.push(info),
                Err(error_msg) => error_repos.push(error_msg),
            }
        }
        (succ_repos, error_repos)
    });

    match error_repos.len() {
        0 => {
            let mut result = StyleMessage::ops_success("sync");
            // show track status
            if !silent {
                // show stash status
                if succ_repos.iter().any(|info| !info.stash_status.is_empty()) {
                    result = result.join("\n".into());
                    result = result.join("Stash status:\n".into());
                    for info in &succ_repos {
                        if info.stash_status.is_empty() {
                            continue;
                        }
                        result = result.join(format!("  {}\n", info.stash_status).into());
                    }
                }

                // show track status
                result = result.join("\n".into());
                result = result.join("Track status:\n".into());
                for info in &succ_repos {
                    result = result.join(format!("  {}\n", info.track_status).into());
                }
            }
            Ok(result)
        }
        _ => {
            let msg = StyleMessage::ops_failed("sync", error_repos.len());
            Err(anyhow!(MgitError::OpsError {
                prefix: msg,
                errors: OpsErrors(error_repos),
            }))
        }
    }
}

fn inner_exec(
    input_path: &Path,
    repo_info: &mut RepoInfo,
    stash_mode: &StashMode,
    no_checkout: bool,
    depth: Option<&usize>,
    default_branch: &Option<String>,
    progress: &impl Progress,
) -> anyhow::Result<InnerExecResponse> {
    let full_path = &input_path.join(repo_info.rel_path());

    let mut toml_repo = repo_info.toml_repo.to_owned();
    let mut owned_repo_info = repo_info.to_owned();
    let repo_info = &mut owned_repo_info;
    // make repo directory and skip clone the repository
    std::fs::create_dir_all(full_path)
        .with_context(|| format!("create dir {} failed.", full_path.to_str().unwrap()))?;

    let mut stash_mode = stash_mode.to_owned();
    let is_repo_none = git::is_repository(full_path.as_path()).is_err();
    // if repository not found, create new one
    if is_repo_none {
        // use --hard
        stash_mode = StashMode::Hard;

        // git init when dir exist
        exec_init(input_path, repo_info, progress)?;
        // git remote add url
        exec_add_remote(input_path, repo_info, progress)?;
    } else {
        let remote_url = repo_info.toml_repo.remote.as_ref().unwrap();
        git::update_remote_url(full_path, remote_url)?;
    }

    // use default branch when branch is null
    if repo_info.toml_repo.branch.is_none() {
        toml_repo.branch = default_branch.to_owned();
        repo_info.toml_repo = &toml_repo;
    }

    // fetch
    exec_fetch(input_path, repo_info, depth, progress)?;

    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_info.toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    // check remote-ref valid
    git::is_remote_ref_valid(full_path, remote_ref_str)?;

    let mut exec_response = InnerExecResponse::default();

    match stash_mode {
        StashMode::Normal => {
            // try stash → checkout → reset → stash pop
            if !no_checkout {
                // stash
                let stash_response = exec_stash(input_path, repo_info, progress)?;

                // checkout
                let mut result = exec_checkout(input_path, repo_info, progress, false);

                if result.is_ok() {
                    // reset --hard
                    result = exec_reset(input_path, repo_info, progress, ResetType::Hard);
                }

                // stash pop, whether checkout succ or failed, whether reset succ or failed
                if matches!(stash_response, InnerStashResponse::Stash(_)) {
                    let _ = exec_stash_pop(input_path, repo_info, progress);
                }
                result
            } else {
                // reset --soft
                exec_reset(input_path, repo_info, progress, ResetType::Soft)
            }
        }

        StashMode::Stash => {
            // stash with `--stash` option, maybe return error if need to initial commit
            let stash_response = exec_stash(input_path, repo_info, progress)?;

            let mut result: Result<(), anyhow::Error> = Ok(());
            let mut reset_type = ResetType::Mixed;

            // checkout
            if !no_checkout {
                result = exec_checkout(input_path, repo_info, progress, true);
                reset_type = ResetType::Hard;
            }

            if result.is_ok() {
                result = exec_reset(input_path, repo_info, progress, reset_type);
            }

            if matches!(stash_response, InnerStashResponse::Stash(_)) {
                // undo if checkout failed or reset failed
                if let Err(e) = result {
                    // if reset failed, pop stash if stash something this time
                    let _ = exec_stash_pop(input_path, repo_info, progress);
                    return Err(e);
                }

                // save stash message
                exec_response.stash = Some(stash_response);
            }
            result
        }

        StashMode::Hard => {
            // clean
            if !is_repo_none {
                exec_clean(input_path, repo_info, progress)?;
            }

            // checkout
            if !no_checkout {
                exec_checkout(input_path, repo_info, progress, true)?;
            }

            // reset --hard
            exec_reset(input_path, repo_info, progress, ResetType::Hard)
        }
    }?;

    match repo_info.toml_repo.sparse.as_ref() {
        Some(dirs) => git::sparse_checkout_set(&full_path, dirs),
        None => git::sparse_checkout_disable(&full_path),
    }?;

    Ok(exec_response)
}

fn exec_init(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    progress.repo_info(repo_info, "initialize...".into());
    git::init(input_path.join(repo_info.rel_path()))
}

fn exec_add_remote(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    progress.repo_info(repo_info, "add remote...".into());

    let full_path = input_path.join(repo_info.rel_path());
    let url = repo_info.toml_repo.remote.as_ref().unwrap();
    git::add_remote_url(full_path, url)
}

fn exec_clean(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    progress.repo_info(repo_info, "clean...".into());

    let full_path = input_path.join(repo_info.rel_path());
    git::clean(full_path)
}

fn exec_reset(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
    reset_type: ResetType,
) -> anyhow::Result<()> {
    progress.repo_info(repo_info, "reset...".into());

    let full_path = input_path.join(repo_info.rel_path());
    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_info.toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    let reset_type = match reset_type {
        ResetType::Soft => "--soft",
        ResetType::Mixed => "--mixed",
        ResetType::Hard => "--hard",
    };

    git::reset(&full_path, reset_type, remote_ref_str)
}

fn exec_stash(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
) -> Result<InnerStashResponse, anyhow::Error> {
    progress.repo_info(repo_info, "stash...".into());

    let full_path = input_path.join(repo_info.rel_path());
    let msg = git::stash(full_path)?;

    let response = match msg.find("WIP") {
        Some(idx) => InnerStashResponse::Stash(msg.trim()[idx..].to_string()),
        None => InnerStashResponse::None,
    };
    Ok(response)
}

fn exec_stash_pop(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
) -> Result<String, anyhow::Error> {
    progress.repo_info(repo_info, "pop stash...".into());

    let full_path = input_path.join(repo_info.rel_path());
    git::stash_pop(full_path)
}

fn exec_checkout(
    input_path: &Path,
    repo_info: &RepoInfo,
    progress: &impl Progress,
    force: bool,
) -> anyhow::Result<()> {
    progress.repo_info(repo_info, "checkout...".into());

    let full_path = input_path.join(repo_info.rel_path());
    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_info.toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };
    let branch = match remote_ref {
        RemoteRef::Commit(commit) => format!("commits/{}", &commit[..7]),
        RemoteRef::Tag(tag) => format!("tags/{}", tag),
        RemoteRef::Branch(_) => repo_info
            .toml_repo
            .branch
            .clone()
            .unwrap_or("invalid-branch".to_string()),
    };

    // don't need to checkout if current branch is the branch
    if let Ok(current_branch) = git::get_current_branch(full_path.as_path()) {
        if branch == current_branch {
            return Ok(());
        }
    }

    let suffix = StyleMessage::git_checking_out(&branch);
    progress.repo_info(repo_info, suffix);

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
