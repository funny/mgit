mod cli;
mod commands;
mod utils;

use clap::Parser;
use color_eyre::eyre::eyre;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use mgit::ops;
use mgit::utils::error::MgitResult;
use mgit::utils::StyleMessage;

use crate::cli::{Cli, Commands};
use crate::utils::logger::TERM_LOGGER;
use crate::utils::progress::MultiProgress;

fn main() -> color_eyre::Result<()> {
    init_log();

    let progress = MultiProgress::default();
    let cli = Cli::parse();
    let result: MgitResult = match cli.command {
        Commands::Init(options) => ops::init_repo(options.into()),
        Commands::Snapshot(options) => ops::snapshot_repo(options.into()),
        Commands::Fetch(options) => ops::fetch_repos(options.into(), progress),
        Commands::Sync(options) => ops::sync_repo(options.into(), progress),
        Commands::Clean(options) => ops::clean_repo(options.into()),
        Commands::ListFiles(options) => match ops::list_files(options.into()) {
            Ok(files) => {
                println!("{}", files.join("\n"));
                Ok(StyleMessage::default())
            }
            Err(e) => Err(e),
        },
        Commands::Track(options) => ops::track(options.into(), progress),
        Commands::LogRepos(options) => match ops::log_repos(options.into()) {
            Ok(repo_logs) => {
                repo_logs.into_iter().for_each(|repo_log| match repo_log {
                    Ok(repo_log) => {
                        println!("{}", repo_log);
                    }
                    Err(e) => eprintln!("{:?}", eyre!(e)),
                });
                Ok(StyleMessage::default())
            }
            Err(e) => Err(e),
        },
    };

    println!("{}", result.map_err(|e| eyre!(e))?);
    Ok(())
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
