use super::commands;
use clap::{Parser, Subcommand, ArgAction};

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
        path: Option<String>
    },

    /// Sync git repos
    Sync {},

    /// Fetch git repos
    Fetch {},

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

    // handle commands
    match args.command {
        Commands::Init { path } => {
            commands::init::exec(path);
        },

        Commands::Sync {} => {
            commands::sync::exec();
        },

        Commands::Fetch {} => {
            commands::fetch::exec();
        },

        Commands::Clean { force } => {
            commands::clean::exec(force);
        },
    };
}
