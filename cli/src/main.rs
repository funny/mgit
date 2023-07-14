mod cli;
mod commands;
mod utils;

use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use mgit::ops;

use crate::cli::{Cli, Commands};
use crate::utils::logger::TERM_LOGGER;
use crate::utils::progress::MultiProgress;

fn main() {
    init_log();

    let progress = MultiProgress::default();
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(options) => ops::init_repo(options.into()),
        Commands::Snapshot(options) => ops::snapshot_repo(options.into()),
        Commands::Fetch(options) => ops::fetch_repos(options.into(), progress),
        Commands::Sync(options) => ops::sync_repo(options.into(), progress),
        Commands::Clean(options) => ops::clean_repo(options.into()),
        Commands::ListFiles(options) => ops::list_files(options.into()),
        Commands::Track(options) => ops::track(options.into(), progress),
    }
}

fn init_log() {
    mgit::utils::logger::set_logger(&TERM_LOGGER);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("stdout", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
