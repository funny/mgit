use clap::Args;
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, LogReposOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Log git repos
pub(crate) struct LogReposCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,

    /// Labels for log repos
    #[arg(long)]
    labels: Option<Vec<String>>,
}

impl CliCommand for LogReposCommand {
    async fn exec(self) -> MgitResult<()> {
        let repo_logs = ops::log_repos(self.into()).await?;

        for repo_log in repo_logs {
            match repo_log {
                Ok(repo_log) => println!("{}", repo_log),
                Err(e) => eprintln!("{:?}", e),
            };
        }

        Ok(())
    }
}

impl From<LogReposCommand> for LogReposOptions {
    fn from(value: LogReposCommand) -> Self {
        LogReposOptions::new(value.path, value.config, Some(value.thread), value.labels)
    }
}
