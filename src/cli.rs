use super::commands;
use clap::{ArgAction, Parser, Subcommand};
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
    Sync {},

    /// Fetch git repos
    Fetch {
        /// The init directory
        path: Option<String>,
    },

    /// Clean unused git repos
    Clean {
        /// force remove git repos without prompt
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
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

        Commands::Sync {} => {
            commands::sync::exec();
        }

        Commands::Fetch { path } => {
            commands::fetch::exec(path);
        }

        Commands::Clean { force } => {
            commands::clean::exec(force);
        }
    };
}
