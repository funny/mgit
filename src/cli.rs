use super::commands;
use std::path::PathBuf;

use clap::{error::ErrorKind, ArgAction, CommandFactory, Parser, Subcommand};

// ========================================
// main
// ========================================

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    propagate_version = true,
    arg_required_else_help(true)
)]

struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    },

    /// Fetch git repos
    Fetch {
        /// The init directory
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
    },

    /// Clean unused git repos
    Clean {
        /// The init directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },

    /// Track remote branch
    Track {
        /// The init directory
        path: Option<String>,

        /// Use specified config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

// ========================================
// main
// ========================================

pub fn main() {
    let args = Cli::parse();

    // handle commands
    match args.command {
        Commands::Init { path, force } => {
            commands::snapshot::exec(path, None, commands::SnapshotType::Branch, force, None);
        }

        Commands::Snapshot {
            path,
            config,
            branch,
            force,
            ignore,
        } => {
            let snapshot_type = match branch {
                true => commands::SnapshotType::Branch,
                false => commands::SnapshotType::Commit,
            };

            commands::snapshot::exec(path, config, snapshot_type, force, ignore);
        }

        Commands::Sync {
            path,
            config,
            stash,
            hard,
            thread,
            silent,
            no_track,
            no_checkout,
        } => {
            let stash_mode = match (stash, hard) {
                (false, false) => commands::StashMode::Normal,
                (true, false) => commands::StashMode::Stash,
                (false, true) => commands::StashMode::Hard,
                _ => {
                    let mut cmd = Cli::command();
                    cmd.error(
                        ErrorKind::ArgumentConflict,
                        "'--stash' and '--hard' can't be used together.",
                    )
                    .exit();
                }
            };
            commands::sync::exec(
                path,
                config,
                stash_mode,
                thread,
                silent,
                no_track,
                no_checkout,
            );
        }

        Commands::Fetch {
            path,
            config,
            thread,
            silent,
        } => {
            commands::fetch::exec(path, config, thread, silent);
        }

        Commands::Clean { path, config } => {
            commands::clean::exec(path, config);
        }

        Commands::Track { path, config } => {
            commands::track::exec(path, config);
        }
    };
}
