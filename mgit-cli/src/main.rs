use clap::Parser;
use mgit::error::MgitResult;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::{Cli, Commands};
use crate::commands::CliCommand;

mod cli;
mod commands;
mod term;

#[tokio::main]
async fn main() {
    init_log();

    let cli = Cli::parse();
    let result: MgitResult = match cli.command {
        Commands::Init(cmd) => cmd.exec().await,
        Commands::Snapshot(cmd) => cmd.exec().await,
        Commands::Fetch(cmd) => cmd.exec().await,
        Commands::Sync(cmd) => cmd.exec().await,
        Commands::Clean(cmd) => cmd.exec().await,
        Commands::ListFiles(cmd) => cmd.exec().await,
        Commands::Track(cmd) => cmd.exec().await,
        Commands::LogRepos(cmd) => cmd.exec().await,
        Commands::NewRemoteBranch(cmd) => cmd.exec().await,
        Commands::DelRemoteBranch(cmd) => cmd.exec().await,
        Commands::NewTag(cmd) => cmd.exec().await,
    };

    match result {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1)
        }
    }
}

fn init_log() {
    console::set_colors_enabled(true);
    console::set_colors_enabled_stderr(true);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();
}
