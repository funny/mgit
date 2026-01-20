use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, FetchOptions};

use crate::commands::CliCommand;
use crate::term::progress::MultiProgress;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Fetch git repos
pub(crate) struct FetchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// The number of thread to use, default is 4
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    pub thread: usize,

    /// enable silent mode
    #[arg(long, action = ArgAction::SetTrue)]
    pub silent: bool,

    /// limit fetching to the specified number of commits
    #[arg(short, long, value_name = "NUMBER")]
    pub depth: Option<usize>,

    /// Ignore specified repositories to fetch
    #[arg(long)]
    ignore: Option<Vec<String>>,

    /// Labels for fetch
    #[arg(long)]
    labels: Option<Vec<String>>,
}

impl CliCommand for FetchCommand {
    async fn exec(self) -> MgitResult<()> {
        let progress = MultiProgress::default();
        let msg = ops::fetch_repos(self.into(), progress).await?;
        println!("{}", msg);
        Ok(())
    }
}

impl From<FetchCommand> for FetchOptions {
    fn from(value: FetchCommand) -> Self {
        FetchOptions::new(
            value.path,
            value.config,
            Some(value.thread),
            Some(value.silent),
            value.depth,
            value.ignore,
            value.labels,
        )
    }
}
