use clap::{ArgAction, Args};
use mgit::ops::SyncOptions as CoreSyncOptions;
use mgit::option::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct SyncOptions {
    #[clap(flatten)]
    base: BaseOptions,

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

impl From<SyncOptions> for CoreOptions {
    fn from(value: SyncOptions) -> Self {
        CoreOptions::new_sync_options(
            value.base.path,
            value.base.config,
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
