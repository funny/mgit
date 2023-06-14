use clap::Args;
use mgit::ops::CleanOptions;
use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Clean unused git repos
pub(crate) struct CleanCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

impl From<CleanCommand> for CleanOptions {
    fn from(value: CleanCommand) -> Self {
        CleanOptions::new(value.path, value.config)
    }
}
