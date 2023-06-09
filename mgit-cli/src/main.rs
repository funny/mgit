mod cli;
mod options;

use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::Config;
use mgit::ops;
use mgit::options::CoreOptions;

use crate::cli::{Cli, Commands};

fn main() {
    init_log();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init(options) => ops::init_repo(CoreOptions::from(options)),
        Commands::Snapshot(options) => ops::snapshot_repo(CoreOptions::from(options)),
        Commands::Fetch(options) => ops::fetch_repos(CoreOptions::from(options)),
        Commands::Sync(options) => ops::sync_repo(CoreOptions::from(options)),
        Commands::Clean(options) => ops::clean_repo(CoreOptions::from(options)),
        Commands::ListFiles(options) => ops::list_files(CoreOptions::from(options)),
        Commands::Track(options) => ops::track(CoreOptions::from(options)),
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
