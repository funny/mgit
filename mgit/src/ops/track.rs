use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::config::{repos_to_map_with_ignore, MgitConfig, RepoConfig};
use crate::error::MgitError;
use crate::error::MgitResult;
use crate::git;
use crate::git::RemoteRef;

use crate::utils::current_dir;
use crate::utils::progress::{Progress, RepoInfo};
use crate::utils::StyleMessage;

/// Default number of concurrent operations
const DEFAULT_THREAD_COUNT: usize = 4;

pub struct TrackOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub ignore: Option<Vec<String>>,
}

impl TrackOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => current_dir(),
        };
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            ignore,
        }
    }
}

#[must_use]
pub async fn track(
    options: TrackOptions,
    progress: impl Progress + 'static,
) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let ignore = options.ignore.as_ref();

    tracing::info!("Track status:");
    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        return Err(crate::error::MgitError::DirNotFound { path: path.clone() });
    }
    // check if .gitrepos exists
    if !config_path.is_file() {
        return Err(crate::error::MgitError::ConfigFileNotFound {
            path: config_path.clone(),
        });
    }
    // load config file(like .gitrepos)
    let mgit_config =
        MgitConfig::load(config_path).ok_or(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::other("Failed to load config"),
        })?;

    // handle track
    let repo_configs = if let Some(repos) = mgit_config.repos {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to track"));
    };

    let default_branch = mgit_config.default_branch;

    // retain repos exclude ignore repositories
    let repos_map = repos_to_map_with_ignore(repo_configs, ignore, None);

    progress.on_batch_start(repos_map.len());

    let semaphore = Arc::new(Semaphore::new(DEFAULT_THREAD_COUNT));
    let mut join_set = JoinSet::new();
    let counter = std::sync::atomic::AtomicUsize::new(1);
    let counter = Arc::new(counter);

    let base_path = path.clone();
    let default_branch = Arc::new(default_branch);

    for (id, repo_config) in repos_map {
        let permit =
            Arc::clone(&semaphore)
                .acquire_owned()
                .await
                .map_err(|_| MgitError::OpsError {
                    message: "Failed to acquire semaphore permit for track operation".to_string(),
                })?;
        let counter = counter.clone();
        let progress = progress.clone();
        let base_path = base_path.clone();
        let default_branch = default_branch.clone();
        let repo_config = repo_config.clone();

        join_set.spawn(async move {
            let _permit = permit;
            let index = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let on_repo_update = RepoInfo::new(id, index, &repo_config);

            progress.on_repo_start(&on_repo_update, "tracking repo".into());

            match set_tracking_remote_branch(&base_path, &repo_config, &default_branch).await {
                Ok(msg) => {
                    progress.on_repo_update(&on_repo_update, "tracking".into());
                    progress.on_repo_success(&on_repo_update, msg);
                }
                Err(e) => {
                    progress.on_repo_error(&on_repo_update, format!("failed: {}", e).into());
                }
            }
        });
    }

    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            tracing::error!("Task panicked or cancelled: {}", e);
        }
    }

    progress.on_batch_finish();

    Ok(StyleMessage::new())
}

pub async fn set_tracking_remote_branch(
    input_path: impl AsRef<Path>,
    repo_config: &RepoConfig,
    default_branch: &Option<String>,
) -> MgitResult<StyleMessage> {
    let rel_path = repo_config
        .local
        .as_ref()
        .ok_or_else(|| MgitError::OpsError {
            message: "Repository config missing 'local' field".to_string(),
        })?;
    let full_path = input_path.as_ref().join(rel_path);

    // get local current branch
    let local_branch = git::get_current_branch(full_path.as_path()).await?;

    let mut repo_config = repo_config.to_owned();
    // use default branch when branch is null
    if repo_config.branch.is_none() {
        repo_config.branch = default_branch.to_owned();
    }

    // priority: commit/tag/branch(default-branch)
    let remote_ref = repo_config.get_remote_ref(full_path.as_path()).await?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(r) | RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };
    let remote_desc = match remote_ref {
        RemoteRef::Commit(commit) => commit[..7].to_string(),
        RemoteRef::Tag(r) | RemoteRef::Branch(r) => r,
    };

    if repo_config.commit.is_some() || repo_config.tag.is_some() {
        let res = StyleMessage::git_untracked(rel_path, &remote_desc);
        return Ok(res);
    }

    git::set_tracking_remote_branch(
        full_path,
        rel_path,
        local_branch,
        remote_ref_str,
        remote_desc,
    )
    .await
}
