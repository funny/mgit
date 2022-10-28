use std::path::PathBuf;

use super::commands;
use clap::{error::ErrorKind, ArgAction, CommandFactory, Parser, Subcommand};
use git2;

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

        /// force remove git repos without prompt
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
    },

    /// Sync git repos
    Sync {
        /// The sync directory
        path: Option<String>,

        /// use custom config file
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// stash local changes after sync
        #[arg(long, action = ArgAction::SetTrue)]
        stash: bool,

        /// discard local changes after sync
        #[arg(long, action = ArgAction::SetTrue)]
        hard: bool,

        /// sets the number of threads to be used
        #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
        thread: usize,
    },

    /// Fetch git repos
    Fetch {
        /// The init directory
        path: Option<String>,

        /// use custom config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// sets the number of threads to be used
        #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
        thread: usize,
    },

    /// Clean unused git repos
    Clean {
        /// The init directory
        path: Option<String>,

        /// use custom config file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

// ========================================
// main
// ========================================

pub fn main() {
    let args = Cli::parse();

    // set git options
    unsafe {
        git2::opts::set_verify_owner_validation(false)
            .expect("Failed to call git2::opts::set_verify_owner_validation");
    }

    // handle commands
    match args.command {
        Commands::Init { path, force } => {
            commands::init::exec(path, force);
        }

        Commands::Sync {
            path,
            config,
            stash,
            hard,
            thread,
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
            commands::sync::exec(path, config, stash_mode, thread);
        }

        Commands::Fetch {
            path,
            config,
            thread,
        } => {
            commands::fetch::exec(path, config, thread);
        }

        Commands::Clean { path, config } => {
            commands::clean::exec(path, config);
        }
    };
}
