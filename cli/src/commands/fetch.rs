use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, FetchOptions};
use mgit::utils::error::MgitResult;

use crate::utils::progress::MultiProgress;
use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct FetchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,

    /// Do not report git status
    #[arg(long, action = ArgAction::SetTrue)]
    silent: bool,

    /// Deepen history of shallow clone
    #[arg(short, long, value_name = "NUMBER")]
    depth: Option<usize>,

    /// Ignore specified repositories for fetch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for FetchCommand {
    fn exec(self) -> MgitResult {
        let progress = MultiProgress::default();
        ops::fetch_repos(self.into(), progress)
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
        )
    }
}
