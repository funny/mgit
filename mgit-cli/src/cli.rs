use clap::{command, Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::options::*;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    propagate_version = true,
    arg_required_else_help(true)
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Init git repos
    Init(InitOptions),

    /// Fetch git repos
    Fetch(FetchOptions),

    /// Snapshot git repos
    Snapshot(SnapshotOptions),

    /// Sync git repos
    Sync(SyncOptions),

    /// Clean unused git repos
    Clean(CleanOptions),

    /// List tree files
    #[command(name = "ls-files")]
    ListFiles(ListFilesOptions),

    /// Track remote branch
    Track(TrackOptions),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct BaseOptions {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
