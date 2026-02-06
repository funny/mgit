use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::config::{cmp_local_remote, repos_to_map_with_ignore, MgitConfig};
use crate::error::{
    AcquirePermitFailedSnafu, BranchReferenceRequiredSnafu, MgitResult, NoRemoteConfiguredSnafu,
};
use crate::git;
use crate::git::RemoteRef;
use crate::utils::cmd::{retry, DEFAULT_RETRY_COUNT, DEFAULT_RETRY_DELAY_MS};
use crate::utils::path::PathExtension;
use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::style_message::StyleMessage;
use crate::utils::{cmd, current_dir};

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
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => current_dir(),
        };
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

#[must_use]
pub async fn fetch_repos(
    options: FetchOptions,
    progress: impl Progress + 'static,
) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let thread_count = options.thread_count;
    let silent = options.silent;
    let depth = options.depth;
    let ignore = options.ignore.as_ref();

    tracing::info!(message = %StyleMessage::ops_start("fetch repos", path));

    if !config_path.is_file() {
        return Err(crate::error::MgitError::ConfigFileNotFound {
            path: config_path.clone(),
        });
    }

    let mgit_config =
        MgitConfig::load(config_path).ok_or(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::other("Failed to load config"),
        })?;

    let repo_configs = if let Some(repos) = mgit_config.repos {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to fetch"));
    };

    let default_branch = mgit_config.default_branch;
    let repos_map = repos_to_map_with_ignore(repo_configs, ignore, options.labels.as_ref());

    progress.on_batch_start(repos_map.len());

    let semaphore = Arc::new(Semaphore::new(thread_count));
    let mut join_set = JoinSet::new();
    let counter = std::sync::atomic::AtomicUsize::new(1);
    let counter = Arc::new(counter);

    // We need to clone path for each task
    let base_path = path.clone();
    let default_branch = Arc::new(default_branch);

    for (id, repo_config) in repos_map {
        let permit = Arc::clone(&semaphore).acquire_owned().await.map_err(|_| {
            AcquirePermitFailedSnafu {
                message: "Failed to acquire semaphore permit for parallel execution".to_string(),
            }
            .build()
        })?;
        let counter = Arc::clone(&counter);
        let progress = progress.clone();
        let base_path = base_path.clone();
        let default_branch = Arc::clone(&default_branch);
        let repo_config = repo_config.clone();

        join_set.spawn(async move {
            let _permit = permit; // Hold permit until task finishes
            let index = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let on_repo_update = RepoInfo::new(id, index, &repo_config);

            progress.on_repo_start(&on_repo_update, "waiting...".into());

            // execute fetch command
            let exec_res = inner_exec(&base_path, &on_repo_update, depth.as_ref(), &progress).await;

            match exec_res {
                Ok(_) => {
                    let msg = if silent {
                        StyleMessage::new()
                    } else {
                        cmp_local_remote(&base_path, &repo_config, &default_branch, false)
                            .await
                            .unwrap_or_else(|_| StyleMessage::new())
                    };
                    progress.on_repo_success(&on_repo_update, msg);
                    Ok(())
                }
                Err(e) => {
                    progress.on_repo_error(&on_repo_update, StyleMessage::new());
                    Err(StyleMessage::git_error(
                        repo_config
                            .local
                            .as_ref()
                            .map(|p| p.display_path())
                            .unwrap_or_else(|| id.to_string()),
                        &e,
                    ))
                }
            }
        });
    }

    let mut errors = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(Err(e)) = res {
            errors.push(e);
        }
    }

    progress.on_batch_finish();

    if errors.is_empty() {
        Ok(StyleMessage::ops_success("fetch"))
    } else {
        let msg = StyleMessage::ops_failed("fetch", errors.len());
        Err(crate::error::MgitError::OpsError {
            message: format!("{}\nErrors:\n{:?}", msg, errors),
        })
    }
}

async fn inner_exec(
    input_path: impl AsRef<Path>,
    on_repo_update: &RepoInfo<'_>,
    depth: Option<&usize>,
    progress: &impl Progress,
) -> MgitResult<()> {
    let full_path = input_path.as_ref().join(on_repo_update.rel_path());
    let remote_url = on_repo_update.repo_config.remote.as_ref().ok_or_else(|| {
        NoRemoteConfiguredSnafu {
            path: full_path.clone(),
        }
        .build()
    })?;

    git::update_remote_url(&full_path, remote_url).await?;
    exec_fetch(input_path, on_repo_update, depth, progress).await
}

pub async fn exec_fetch(
    input_path: impl AsRef<Path>,
    on_repo_update: &RepoInfo<'_>,
    depth: Option<&usize>,
    progress: &impl Progress,
) -> MgitResult<()> {
    let full_path = input_path.as_ref().join(on_repo_update.rel_path());

    let remote_name: String = on_repo_update
        .repo_config
        .get_remote_name(full_path.as_path())
        .await?;

    // Build args without using String -> &str conversions where possible
    let mut args: Vec<String> = vec!["fetch".to_string(), remote_name];

    if let Some(depth) = depth {
        let remote_ref = on_repo_update
            .repo_config
            .get_remote_ref(full_path.as_path())
            .await?;
        match remote_ref {
            RemoteRef::Commit(commit) => {
                args.push(commit);
            }
            RemoteRef::Tag(tag) => {
                args.push("tag".to_string());
                args.push(tag);
                args.push("--no-tags".to_string());
            }
            RemoteRef::Branch(_) => {
                let branch = on_repo_update.repo_config.branch.as_ref().ok_or_else(|| {
                    BranchReferenceRequiredSnafu {
                        message: "Branch reference required for branch remote ref".to_string(),
                    }
                    .build()
                })?;
                args.push(branch.clone());
            }
        };

        args.push("--depth".to_string());
        args.push(depth.to_string());
    }

    args.push("--prune".to_string());
    args.push("--recurse-submodules=on-demand".to_string());
    args.push("--progress".to_string());

    retry(
        DEFAULT_RETRY_COUNT,
        Duration::from_millis(DEFAULT_RETRY_DELAY_MS),
        || async {
            let mut command = Command::new("git");
            command.args(&args).current_dir(&full_path);
            cmd::exec_cmd_with_progress(on_repo_update, &mut command, progress).await
        },
    )
    .await
}
