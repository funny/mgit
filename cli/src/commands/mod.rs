use clap::{ArgAction, ArgMatches, Parser, Subcommand};
use std::path::PathBuf;

mod clean;
mod fetch;
mod init;
mod ls_files;
mod snapshot;
mod sync;
mod track;

#[derive(PartialEq, Clone)]
pub enum StashMode {
    Normal,
    Stash,
    Hard,
}

pub enum ResetType {
    Soft,
    Mixed,
    Hard,
}

pub enum SnapshotType {
    Commit,
    Branch,
}

#[derive(Clone)]
pub enum RemoteRef {
    Commit(String),
    Tag(String),
    Branch(String),
}

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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Init git repos
    Init {
        /// The init directory
        path: Option<String>,

        /// Force remove git repos without prompt
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
    },

    /// Snapshot git repos
    Snapshot {
        /// The init directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// snapshot by branch
        #[arg(long, action = ArgAction::SetTrue)]
        branch: bool,

        /// Force remove git repos without prompt
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,

        /// Ignore specified repositories for snapshot
        #[arg(long)]
        ignore: Option<Vec<String>>,
    },

    /// Sync git repos
    Sync {
        /// The sync directory
        path: Option<String>,

        /// Use specified config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

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
    },

    /// Fetch git repos
    Fetch {
        /// The fetch directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,

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
    },

    /// Clean unused git repos
    Clean {
        /// The clean directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },

    /// Track remote branch
    Track {
        /// The track directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Ignore specified repositories for track
        #[arg(long)]
        ignore: Option<Vec<String>>,
    },

    /// List tree
    LsFiles {
        /// The list directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

pub fn builtin_exec(cmd: &str) -> Option<fn(&ArgMatches)> {
    let f = match cmd {
        "init" => init::exec,
        "snapshot" => snapshot::exec,
        "fetch" => fetch::exec,
        "sync" => sync::exec,
        "track" => track::exec,
        "clean" => clean::exec,
        "ls-files" => ls_files::exec,
        _ => return None,
    };
    Some(f)
}
