mod cli;
mod options;

use crate::cli::{Cli, Commands};
use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::Config;
use mgit::ops;
use mgit::option::CoreOptions;

fn main() {
    init_log();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init(options) => ops::init_repo(Into::<CoreOptions>::into(options)),
        Commands::Fetch(options) => ops::fetch_repos(Into::<CoreOptions>::into(options)),
        Commands::Snapshot(options) => ops::snapshot_repo(Into::<CoreOptions>::into(options)),
        Commands::Sync(options) => ops::sync_repo(Into::<CoreOptions>::into(options)),
        Commands::Clean(options) => ops::clean_repo(Into::<CoreOptions>::into(options)),
        Commands::ListFiles(options) => ops::list_files(Into::<CoreOptions>::into(options)),
        Commands::Track(options) => ops::track(Into::<CoreOptions>::into(options)),
    }
}

fn init_log() {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("stdout", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
