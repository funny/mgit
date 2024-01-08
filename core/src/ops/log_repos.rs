use std::env;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};

use crate::core::git::log_current;
use crate::core::repos::{load_config, TomlConfig};
use crate::utils::error::{MgitError, MgitResult};
use crate::utils::path::PathExtension;
use crate::utils::{logger, StyleMessage};

pub struct LogReposOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub thread_count: usize,
}

impl LogReposOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread_count: Option<usize>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            thread_count: thread_count.unwrap_or(4),
        }
    }

    pub fn validate(self) -> MgitResult<(PathBuf, TomlConfig, usize)> {
        let LogReposOptions {
            path,
            config_path,
            thread_count,
        } = self;

        // if directory doesn't exist, return
        if !path.is_dir() {
            return Err(anyhow!(MgitError::DirNotFound(
                StyleMessage::dir_not_found(&path)
            )));
        }

        // check if .gitrepos exists
        if !config_path.is_file() {
            return Err(anyhow!(MgitError::ConfigFileNotFound(
                StyleMessage::config_file_not_found()
            )));
        }

        // load config file(like .gitrepos)
        let Some(toml_config) = load_config(&config_path) else {
            return Err(anyhow!(MgitError::LoadConfigFailed));
        };

        Ok((path, toml_config, thread_count))
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

pub fn log_repos(options: LogReposOptions) -> MgitResult<Vec<MgitResult<RepoLog>>> {
    let (path, toml_config, thread_count) = options.validate()?;

    logger::info(StyleMessage::ops_start("log repos", &path));

    let toml_repos = toml_config.repos.unwrap_or_default();

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()?;
    let repo_logs = thread_pool.install(|| {
        toml_repos
            .par_iter()
            .map(|toml_repo| {
                let local = toml_repo.local.as_ref().unwrap().to_string();
                let remote = toml_repo.remote.as_ref().unwrap().to_string();
                let rel_path = path.join(&local);
                let log = log_current(rel_path)?;
                let mut logs = log.trim_matches('"').split('\n');
                let sha1 = logs.next().unwrap().trim_matches('"').to_string();
                let author = logs.next().unwrap().trim_matches('"').to_string();
                let date = logs.next().unwrap().trim_matches('"').to_string();
                let log = logs.next().unwrap().trim_matches('"').to_string();
                let repo_log = RepoLog {
                    local: local.display_path(),
                    remote,
                    sha1,
                    author,
                    date,
                    log,
                };
                Ok(repo_log)
            })
            .collect::<Vec<MgitResult<RepoLog>>>()
    });
    Ok(repo_logs)
}
