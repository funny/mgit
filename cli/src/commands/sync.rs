use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, SyncOptions};
use mgit::utils::error::MgitResult;

use crate::utils::progress::MultiProgress;
use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct SyncCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Stash local changes after sync
    #[arg(long, action = ArgAction::SetTrue)]
    stash: bool,

    /// Discard local changes after sync
    #[arg(long, action = ArgAction::SetTrue)]
    hard: bool,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,

    /// Do not report git status
    #[arg(long, action = ArgAction::SetTrue)]
    silent: bool,

    /// Do not track remote branch
    #[arg(long, action = ArgAction::SetTrue)]
    no_track: bool,

    /// Do not checkout branch after sync
    #[arg(long, action = ArgAction::SetTrue)]
    no_checkout: bool,

    /// Deepen history of shallow clone
    #[arg(short, long, value_name = "NUMBER")]
    depth: Option<usize>,

    /// Ignore specified repositories for sync
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for SyncCommand {
    fn exec(self) -> MgitResult {
        let progress = MultiProgress::default();
        ops::sync_repo(self.into(), progress)
    }
}

impl From<SyncCommand> for SyncOptions {
    fn from(value: SyncCommand) -> Self {
        SyncOptions::new(
            value.path,
            value.config,
            Some(value.thread),
            Some(value.silent),
            value.depth,
            value.ignore,
            Some(value.hard),
            Some(value.stash),
            Some(value.no_track),
            Some(value.no_checkout),
        )
    }
}
