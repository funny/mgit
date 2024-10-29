use clap::Parser;
use color_eyre::eyre::eyre;
use log::LevelFilter;

use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

use mgit::utils::error::MgitResult;

use crate::cli::{Cli, Commands};
use crate::commands::CliCommad;
use crate::utils::logger::TERM_LOGGER;

mod cli;
mod commands;
mod utils;

fn main() {
    init_log();

    let cli = Cli::parse();
    let result: MgitResult = match cli.command {
        Commands::Init(cmd) => cmd.exec(),
        Commands::Snapshot(cmd) => cmd.exec(),
        Commands::Fetch(cmd) => cmd.exec(),
        Commands::Sync(cmd) => cmd.exec(),
        Commands::Clean(cmd) => cmd.exec(),
        Commands::ListFiles(cmd) => cmd.exec(),
        Commands::Track(cmd) => cmd.exec(),
        Commands::LogRepos(cmd) => cmd.exec(),
        Commands::NewRemoteBranch(cmd) => cmd.exec(),
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("{}", eyre!(e));
            std::process::exit(1)
        }
    }
}

fn init_log() {
    color_eyre::install().unwrap();
    mgit::utils::logger::set_logger(&TERM_LOGGER);

    console::set_colors_enabled(true);
    console::set_colors_enabled_stderr(true);

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
