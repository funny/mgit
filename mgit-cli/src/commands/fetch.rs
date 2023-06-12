use clap::{ArgAction, Args};
use mgit::ops::FetchOptions;
use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct FetchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,

    /// Do not report git status
    #[arg(long, action = ArgAction::SetTrue)]
    silent: bool,

    /// Deepen history of shallow clone
    #[arg(short, long, value_name = "NUMBER")]
    depth: Option<usize>,

    /// Ignore specified repositories for fetch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl From<FetchCommand> for FetchOptions {
    fn from(value: FetchCommand) -> Self {
        FetchOptions::new(
            value.path,
            value.config,
            Some(value.thread),
            Some(value.silent),
            value.depth,
            value.ignore,
        )
    }
}
