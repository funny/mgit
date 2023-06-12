use clap::Args;
use mgit::ops::ListFilesOptions;
use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// List tree
pub(crate) struct ListFilesCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

impl From<ListFilesCommand> for ListFilesOptions {
    fn from(value: ListFilesCommand) -> Self {
        ListFilesOptions::new(value.path, value.config)
    }
}
