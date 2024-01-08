use clap::Args;
use mgit::ops::LogReposOptions;
use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct LogReposCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,
}

impl From<LogReposCommand> for LogReposOptions {
    fn from(value: LogReposCommand) -> Self {
        LogReposOptions::new(value.path, value.config, Some(value.thread))
    }
}
