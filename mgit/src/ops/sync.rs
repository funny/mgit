use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::config::{cmp_local_remote, repos_to_map_with_ignore, MgitConfig};
use crate::git;
use crate::git::{RemoteRef, ResetType, StashMode};

use crate::error::MgitResult;
use crate::ops::{clean_repo, exec_fetch, set_tracking_remote_branch, CleanOptions};
use crate::utils::path::PathExtension;
use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::style_message::StyleMessage;

/// Internal execution response for sync operations
#[derive(Debug, Default)]
struct SyncExecResponse {
    stash: Option<StashResponse>,
}

/// Stash operation response
#[derive(Debug)]
enum StashResponse {
    None,
    Stash(String),
}

/// Options for synchronizing repositories
#[derive(Debug, Default)]
pub struct SyncOptions {
    /// Base path for repositories
    pub path: PathBuf,
    /// Path to the `.gitrepos` configuration file
    pub config_path: PathBuf,
    /// Number of threads to use for parallel operations
    pub thread_count: usize,
    /// Whether to suppress status output
    pub silent: bool,
    /// Shallow clone depth (None for full clone)
    pub depth: Option<usize>,
    /// List of repository paths to ignore
    pub ignore: Option<Vec<String>>,
    /// List of labels to filter repositories
    pub labels: Option<Vec<String>>,
    /// Whether to discard all local changes (hard reset)
    pub hard: bool,
    /// Whether to stash local changes before sync
    pub stash: bool,
    /// Whether to skip tracking remote branches
    pub no_track: bool,
    /// Whether to skip checking out branches
    pub no_checkout: bool,
}

impl SyncOptions {
    /// Create a new SyncOptions builder with default values
    pub fn builder() -> SyncOptionsBuilder {
        SyncOptionsBuilder::default()
    }

    /// Create new SyncOptions with default values (for backward compatibility)
    ///
    /// # Arguments
    ///
    /// All arguments are optional and will use defaults if not provided.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread_count: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
        labels: Option<Vec<String>>,
        hard: Option<bool>,
        stash: Option<bool>,
        no_track: Option<bool>,
        no_checkout: Option<bool>,
    ) -> Self {
        Self::builder()
            .path(path)
            .config_path(config_path)
            .thread_count(thread_count)
            .silent(silent)
            .depth(depth)
            .ignore(ignore)
            .labels(labels)
            .hard(hard)
            .stash(stash)
            .no_track(no_track)
            .no_checkout(no_checkout)
            .build()
    }
}

/// Builder for SyncOptions
#[derive(Debug, Default)]
pub struct SyncOptionsBuilder {
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    thread_count: Option<usize>,
    silent: Option<bool>,
    depth: Option<usize>,
    ignore: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    hard: Option<bool>,
    stash: Option<bool>,
    no_track: Option<bool>,
    no_checkout: Option<bool>,
}

impl SyncOptionsBuilder {
    /// Set the base path for repositories
    pub fn path(mut self, path: Option<impl AsRef<Path>>) -> Self {
        self.path = path.map(|p| p.as_ref().to_path_buf());
        self
    }

    /// Set the path to the configuration file
    pub fn config_path(mut self, config_path: Option<impl AsRef<Path>>) -> Self {
        self.config_path = config_path.map(|p| p.as_ref().to_path_buf());
        self
    }

    /// Set the number of threads for parallel operations
    pub fn thread_count(mut self, thread_count: Option<usize>) -> Self {
        self.thread_count = thread_count;
        self
    }

    /// Set whether to suppress status output
    pub fn silent(mut self, silent: Option<bool>) -> Self {
        self.silent = silent;
        self
    }

    /// Set the shallow clone depth
    pub fn depth(mut self, depth: Option<usize>) -> Self {
        self.depth = depth;
        self
    }

    /// Set the list of repository paths to ignore
    pub fn ignore(mut self, ignore: Option<Vec<String>>) -> Self {
        self.ignore = ignore;
        self
    }

    /// Set the list of labels to filter repositories
    pub fn labels(mut self, labels: Option<Vec<String>>) -> Self {
        self.labels = labels;
        self
    }

    /// Set whether to discard all local changes (hard reset)
    pub fn hard(mut self, hard: Option<bool>) -> Self {
        self.hard = hard;
        self
    }

    /// Set whether to stash local changes before sync
    pub fn stash(mut self, stash: Option<bool>) -> Self {
        self.stash = stash;
        self
    }

    /// Set whether to skip tracking remote branches
    pub fn no_track(mut self, no_track: Option<bool>) -> Self {
        self.no_track = no_track;
        self
    }

    /// Set whether to skip checking out branches
    pub fn no_checkout(mut self, no_checkout: Option<bool>) -> Self {
        self.no_checkout = no_checkout;
        self
    }

    /// Build the SyncOptions
    pub fn build(self) -> SyncOptions {
        let path = self.path.unwrap_or_else(|| {
            env::current_dir().expect("Failed to get current directory")
        });
        let config_path = self.config_path.unwrap_or_else(|| path.join(".gitrepos"));
        SyncOptions {
            path,
            config_path,
            thread_count: self.thread_count.unwrap_or(4),
            silent: self.silent.unwrap_or(false),
            depth: self.depth,
            ignore: self.ignore,
            labels: self.labels,
            hard: self.hard.unwrap_or(false),
            stash: self.stash.unwrap_or(false),
            no_track: self.no_track.unwrap_or(false),
            no_checkout: self.no_checkout.unwrap_or(false),
        }
    }
}

/// Synchronize repositories according to configuration
///
/// This function reads the `.gitrepos` configuration file and synchronizes all
/// configured repositories to their specified branches, tags, or commits.
///
/// # Arguments
///
/// * `options` - Synchronization options
/// * `progress` - Progress reporter for tracking operation status
///
/// # Returns
///
/// Returns a `StyleMessage` containing the result of the synchronization operation.
pub async fn sync_repo(
    options: SyncOptions,
    progress: impl Progress + 'static + Clone + Send + Sync,
) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let thread_count = options.thread_count;
    let hard = options.hard;
    let stash = options.stash;
    let silent = options.silent;
    let no_track = options.no_track;
    let no_checkout = options.no_checkout;
    let depth = options.depth;
    let ignore = options.ignore.as_ref();

    tracing::info!(message = %StyleMessage::ops_start("sync repos", path).to_plain_text());

    let stash_mode = match (stash, hard) {
        (false, false) => StashMode::Normal,
        (true, false) => StashMode::Stash,
        (false, true) => StashMode::Hard,
        _ => panic!("'--stash' and '--hard' can't be used together."),
    };

    // check if .gitrepos exists
    if !config_path.is_file() {
        return Err(crate::error::MgitError::ConfigFileNotFound {
            path: config_path.clone(),
        });
    }

    // load config file(like .gitrepos)
    let mgit_config =
        MgitConfig::load(config_path).ok_or(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "Failed to load config"),
        })?;

    // remove unused repositories when use '--config' option
    // also if input_path not exists, skip this process
    if stash_mode == StashMode::Hard && path.is_dir() {
        let res = clean_repo(CleanOptions::new(
            Some(path.clone()),
            Some(config_path.clone()),
            options.labels.clone(),
        ))
        .await?;

        tracing::info!(message = %res.to_plain_text());
    }

    // load .gitrepos
    let repo_configs = if let Some(repos) = mgit_config.repos {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to sync"));
    };

    let default_branch = mgit_config.default_branch;

    // retain repos exclude ignore repositories
    let repos_map = repos_to_map_with_ignore(repo_configs, ignore, options.labels.as_ref());
    tracing::info!("Repos count: {}", repos_map.len());
    progress.on_batch_start(repos_map.len());

    let semaphore = Arc::new(Semaphore::new(thread_count));
    let mut join_set = JoinSet::new();
    let counter = std::sync::atomic::AtomicUsize::new(1);
    let counter = Arc::new(counter);

    let base_path = path.clone();
    let default_branch = Arc::new(default_branch);
    let stash_mode = Arc::new(stash_mode);

    struct SuccRepoInfo {
        stash_status: StyleMessage,
        track_status: StyleMessage,
    }

    for (id, repo_config) in repos_map {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let counter = counter.clone();
        let progress = progress.clone();
        let base_path = base_path.clone();
        let default_branch = default_branch.clone();
        let stash_mode = stash_mode.clone();
        let id = id;
        let repo_config = repo_config.clone();

        join_set.spawn(async move {
            let _permit = permit;
            let index = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut repo_info = RepoInfo::new(id, index, &repo_config);

            progress.on_repo_start(&repo_info, "waiting...".into());

            // get compare stat betwwen local and specified commit/tag/branch/
            let mut pre_cmp_msg = StyleMessage::new();
            if !silent {
                let cmp_res =
                    cmp_local_remote(&base_path, &repo_config, &default_branch, false).await;
                pre_cmp_msg = pre_cmp_msg.try_join(cmp_res.ok());
            }

            // execute command according each repo status
            let exec_res = inner_exec(
                &base_path,
                &mut repo_info,
                &stash_mode,
                no_checkout,
                depth.as_ref(),
                &default_branch,
                &progress,
            )
            .await;

            match exec_res {
                Ok(response) => {
                    // if not silent, show compare stat betweent local and remote
                    let msg = if silent {
                        StyleMessage::new()
                    } else {
                        let mut cmp_msg =
                            cmp_local_remote(&base_path, &repo_config, &default_branch, false)
                                .await
                                .unwrap_or(StyleMessage::new());
                        let already_update = cmp_msg.contains("already update to date.");

                        if pre_cmp_msg != cmp_msg && already_update {
                            cmp_msg = cmp_msg.remove("already update to date.");
                            cmp_msg = StyleMessage::git_update_to(cmp_msg);
                        }
                        cmp_msg
                    };

                    // show message in progress bar
                    progress.on_repo_success(&repo_info, msg);

                    // stash status: stash on some commit
                    let mut stash_status = StyleMessage::new();
                    if let Some(StashResponse::Stash(msg)) = response.stash {
                        let repo_rel_path = repo_config.local.as_ref().unwrap().display_path();
                        stash_status =
                            stash_status.join(StyleMessage::git_stash(repo_rel_path, msg));
                    }

                    // track status: track remote branch
                    let mut track_status = StyleMessage::new();
                    if !no_track {
                        let track_res =
                            set_tracking_remote_branch(&base_path, &repo_config, &default_branch)
                                .await;
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
                    progress.on_repo_error(&repo_info, StyleMessage::new());

                    let repo_rel_path = repo_config.local.as_ref().unwrap().display_path();
                    Err(StyleMessage::git_error(repo_rel_path, &e))
                }
            }
        });
    }

    let mut succ_repos = Vec::new();
    let mut error_repos = Vec::new();

    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(Ok(info)) => succ_repos.push(info),
            Ok(Err(e)) => error_repos.push(e),
            Err(e) => {
                // Task panicked or cancelled
                error_repos.push(StyleMessage::new().plain_text(format!("Task failed: {}", e)));
            }
        }
    }

    progress.on_batch_finish();

    if error_repos.is_empty() {
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
    } else {
        let msg = StyleMessage::ops_failed("sync", error_repos.len());
        Err(crate::error::MgitError::OpsError {
            message: format!("{}\nErrors:\n{:?}", msg, error_repos),
        })
    }
}

async fn inner_exec(
    input_path: &Path,
    repo_info: &mut RepoInfo<'_>,
    stash_mode: &StashMode,
    no_checkout: bool,
    depth: Option<&usize>,
    default_branch: &Option<String>,
    progress: &impl Progress,
) -> MgitResult<SyncExecResponse> {
    let full_path = &input_path.join(repo_info.rel_path());

    let mut repo_config = repo_info.repo_config.to_owned();
    // make repo directory and skip clone the repository
    tokio::fs::create_dir_all(full_path)
        .await
        .map_err(|e| crate::error::MgitError::OpsError {
            message: format!("create dir {} failed: {}", full_path.to_str().unwrap(), e),
        })?;

    // Logic for branch update:
    // We modify repo_config copy directly.
    if repo_info.repo_config.branch.is_none() {
        repo_config.branch = default_branch.to_owned();
    }

    // Create local_repo_info based on the (possibly modified) repo_config
    // Since repo_config is local and owned, we can reference it.
    let mut local_repo_info =
        RepoInfo::new(repo_info.id, repo_info.index, &repo_config);
    let current_repo_info = &mut local_repo_info;

    let mut stash_mode = stash_mode.to_owned();
    let is_repo_none = git::is_repository(full_path.as_path()).await.is_err();
    // if repository not found, create new one
    if is_repo_none {
        // use --hard
        stash_mode = StashMode::Hard;

        // git init when dir exist
        exec_init(input_path, current_repo_info, progress).await?;
        // git remote add url
        exec_add_remote(input_path, current_repo_info, progress).await?;
    } else {
        let remote_url = current_repo_info.repo_config.remote.as_ref().unwrap();
        git::update_remote_url(full_path, remote_url).await?;
    }

    // fetch
    exec_fetch(input_path, current_repo_info, depth, progress).await?;

    // priority: commit/tag/branch(default-branch)
    let remote_ref = current_repo_info
        .repo_config
        .get_remote_ref(full_path.as_path())
        .await?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    // check remote-ref valid
    git::is_remote_ref_valid(full_path, remote_ref_str).await?;

    let mut exec_response = SyncExecResponse::default();

    match stash_mode {
        StashMode::Normal => {
            // try stash -> checkout -> reset -> stash pop
            if !no_checkout {
                // stash
                let stash_response =
                    exec_stash(input_path, current_repo_info, progress).await?;

                // checkout
                let mut result =
                    exec_checkout(input_path, current_repo_info, progress, false).await;

                if result.is_ok() {
                    // reset --hard
                    result = exec_reset(
                        input_path,
                        current_repo_info,
                        progress,
                        ResetType::Hard,
                    )
                    .await;
                }

                // stash pop, whether checkout succ or failed, whether reset succ or failed
                if matches!(stash_response, StashResponse::Stash(_)) {
                    let _ = exec_stash_pop(input_path, current_repo_info, progress).await;
                }
                result
            } else {
                // reset --soft
                exec_reset(
                    input_path,
                    current_repo_info,
                    progress,
                    ResetType::Soft,
                )
                .await
            }
        }

        StashMode::Stash => {
            // stash with `--stash` option, maybe return error if need to initial commit
            let stash_response = exec_stash(input_path, current_repo_info, progress).await?;

            let mut result: MgitResult<()> = Ok(());
            let mut reset_type = ResetType::Mixed;

            // checkout
            if !no_checkout {
                result = exec_checkout(input_path, current_repo_info, progress, true).await;
                reset_type = ResetType::Hard;
            }

            if result.is_ok() {
                result = exec_reset(input_path, current_repo_info, progress, reset_type).await;
            }

            if matches!(stash_response, StashResponse::Stash(_)) {
                // undo if checkout failed or reset failed
                if let Err(e) = result {
                    // if reset failed, pop stash if stash something this time
                    let _ = exec_stash_pop(input_path, current_repo_info, progress).await;
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
                exec_clean(input_path, current_repo_info, progress).await?;
            }

            // checkout
            if !no_checkout {
                exec_checkout(input_path, current_repo_info, progress, true).await?;
            }

            // reset --hard
            exec_reset(
                input_path,
                current_repo_info,
                progress,
                ResetType::Hard,
            )
            .await
        }
    }?;

    match current_repo_info.repo_config.sparse.as_ref() {
        Some(dirs) => git::sparse_checkout_set(&full_path, dirs).await,
        None => git::sparse_checkout_disable(&full_path).await,
    }?;

    Ok(exec_response)
}

async fn exec_init(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
) -> MgitResult<()> {
    progress.on_repo_update(repo_info, "initialize...".into());
    git::init(input_path.join(repo_info.rel_path())).await
}

async fn exec_add_remote(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
) -> MgitResult<()> {
    progress.on_repo_update(repo_info, "add remote...".into());

    let full_path = input_path.join(repo_info.rel_path());
    let url = repo_info.repo_config.remote.as_ref().unwrap();
    git::add_remote_url(full_path, url).await
}

async fn exec_clean(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
) -> MgitResult<()> {
    progress.on_repo_update(repo_info, "clean...".into());

    let full_path = input_path.join(repo_info.rel_path());
    git::clean(full_path).await
}

async fn exec_reset(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
    reset_type: ResetType,
) -> MgitResult<()> {
    progress.on_repo_update(repo_info, "reset...".into());

    let full_path = input_path.join(repo_info.rel_path());
    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_info
        .repo_config
        .get_remote_ref(full_path.as_path())
        .await?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    let reset_type = match reset_type {
        ResetType::Soft => "--soft",
        ResetType::Mixed => "--mixed",
        ResetType::Hard => "--hard",
    };

    git::reset(&full_path, reset_type, remote_ref_str).await
}

async fn exec_stash(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
) -> MgitResult<StashResponse> {
    progress.on_repo_update(repo_info, "stash...".into());

    let full_path = input_path.join(repo_info.rel_path());
    let msg = git::stash(full_path).await?;

    let response = match msg.find("WIP") {
        Some(idx) => StashResponse::Stash(msg.trim()[idx..].to_string()),
        None => StashResponse::None,
    };
    Ok(response)
}

async fn exec_stash_pop(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
) -> MgitResult<String> {
    progress.on_repo_update(repo_info, "pop stash...".into());

    let full_path = input_path.join(repo_info.rel_path());
    git::stash_pop(full_path).await
}

async fn exec_checkout(
    input_path: &Path,
    repo_info: &RepoInfo<'_>,
    progress: &impl Progress,
    force: bool,
) -> MgitResult<()> {
    progress.on_repo_update(repo_info, "checkout...".into());

    let full_path = input_path.join(repo_info.rel_path());
    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_info
        .repo_config
        .get_remote_ref(full_path.as_path())
        .await?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };
    let branch = match remote_ref {
        RemoteRef::Commit(commit) => format!("commits/{}", &commit[..7]),
        RemoteRef::Tag(tag) => format!("tags/{}", tag),
        RemoteRef::Branch(_) => repo_info
            .repo_config
            .branch
            .clone()
            .unwrap_or("invalid-branch".to_string()),
    };

    // don't need to checkout if current branch is the branch
    if let Ok(current_branch) = git::get_current_branch(full_path.as_path()).await {
        if branch == current_branch {
            return Ok(());
        }
    }

    let suffix = StyleMessage::git_checking_out(&branch);
    progress.on_repo_update(repo_info, suffix);

    // check if local branch already exists
    let branch_exist = git::local_branch_already_exist(&full_path, &branch).await?;

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

    git::checkout(full_path, &args).await
}
