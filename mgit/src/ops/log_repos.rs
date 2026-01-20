use std::env;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::git::log_current;
use crate::utils::path::PathExtension;
use crate::utils::{label, StyleMessage};

pub struct LogReposOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub thread_count: usize,
    pub labels: Option<Vec<String>>,
}

impl LogReposOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread_count: Option<usize>,
        labels: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            thread_count: thread_count.unwrap_or(4),
            labels,
        }
    }

    pub fn validate(self) -> MgitResult<(PathBuf, MgitConfig, usize, Option<Vec<String>>)> {
        let LogReposOptions {
            path,
            config_path,
            thread_count,
            labels,
        } = self;

        // if directory doesn't exist, return
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
        let Some(mgit_config) = MgitConfig::load(&config_path) else {
            return Err(crate::error::MgitError::LoadConfigFailed {
                source: std::io::Error::new(std::io::ErrorKind::Other, "Failed to load config"),
            });
        };

        Ok((path, mgit_config, thread_count, labels))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoLog {
    pub local: String,
    pub remote: String,
    pub sha1: String,
    pub author: String,
    pub date: String,
    pub log: String,
}

impl Display for RepoLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Local: {}", self.local)?;
        writeln!(f, "Remote: {}", self.remote)?;
        writeln!(f, "Commit: {}", self.sha1)?;
        writeln!(f, "Author: {}", self.author)?;
        writeln!(f, "Date: {}", self.date)?;
        writeln!(f, "Message: {}", self.log)
    }
}

pub async fn log_repos(options: LogReposOptions) -> MgitResult<Vec<MgitResult<RepoLog>>> {
    let (path, mgit_config, thread_count, labels) = options.validate()?;

    tracing::info!(message = %StyleMessage::ops_start("log repos", &path));

    let mut repo_configs = mgit_config.repos.unwrap_or_default();

    if let Some(labels) = labels {
        repo_configs = label::filter(&repo_configs, &labels).cloned().collect();
    }

    let semaphore = Arc::new(Semaphore::new(thread_count));
    let mut join_set = JoinSet::new();
    let base_path = path.clone();

    for repo_config in repo_configs {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let base_path = base_path.clone();

        join_set.spawn(async move {
            let _permit = permit;
            let local = repo_config.local.as_ref().unwrap().to_string();
            let remote = repo_config.remote.as_ref().unwrap().to_string();
            let rel_path = base_path.join(&local);
            let log = log_current(rel_path).await?;
            let mut logs = log.trim_matches('"').split('\n');

            // Need to handle potential split errors if log output format is unexpected
            // But assume git log format is consistent for now as per original code
            let sha1 = logs.next().unwrap_or("").trim_matches('"').to_string();
            let author = logs.next().unwrap_or("").trim_matches('"').to_string();
            let date = logs.next().unwrap_or("").trim_matches('"').to_string();
            let log = logs.next().unwrap_or("").trim_matches('"').to_string();

            let repo_log = RepoLog {
                local: local.display_path(),
                remote,
                sha1,
                author,
                date,
                log,
            };
            Ok(repo_log)
        });
    }

    let mut repo_logs = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(val) => repo_logs.push(val),
            Err(e) => {
                tracing::error!("Task failed: {}", e);
                // We might want to push an Err here to indicate failure
                // Original code returned Vec<MgitResult<RepoLog>>
                // We can construct an MgitError::OpsError
                repo_logs.push(Err(crate::error::MgitError::OpsError {
                    message: format!("Task failed: {}", e),
                }));
            }
        }
    }

    // Sort logs to maintain deterministic output if possible?
    // Original rayon `par_iter().map().collect()` preserves order.
    // `join_set` does NOT preserve order.
    // If order matters (it usually does for UI list), we should sort or use FuturesOrdered.
    // For now, let's sort by local path if possible, but we need to inspect the Result.

    repo_logs.sort_by(|a, b| {
        let empty_string = "".to_string();
        let path_a = a.as_ref().map(|l| &l.local).unwrap_or(&empty_string);
        let path_b = b.as_ref().map(|l| &l.local).unwrap_or(&empty_string);
        path_a.cmp(path_b)
    });

    Ok(repo_logs)
}
