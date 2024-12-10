use clap::{command, Parser, Subcommand};

use crate::commands::*;

#[derive(Parser)]
#[command(
    name = "mgit",
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
    Init(InitCommand),

    /// Fetch git repos
    Fetch(FetchCommand),

    /// Snapshot git repos
    Snapshot(SnapshotCommand),

    /// Sync git repos
    Sync(SyncCommand),

    /// Clean unused git repos
    Clean(CleanCommand),

    /// List tree files
    #[command(name = "ls-files")]
    ListFiles(ListFilesCommand),

    /// Track remote branch
    Track(TrackCommand),

    /// Log git repos
    #[command(name = "log-repos")]
    LogRepos(LogReposCommand),

    /// New Remote Branch
    #[command(name = "new-remote-branch")]
    NewRemoteBranch(NewRemoteBranchCommand),

    /// Delete remote branch
    #[command(name = "del-remote-branch")]
    DelRemoteBranch(DelRemoteBranchCommand),

    /// New tag
    #[command(name = "new-tag")]
    NewTag(NewTagCommand),
}
