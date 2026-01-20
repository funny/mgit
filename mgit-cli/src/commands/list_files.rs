use clap::Args;
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, ListFilesOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// List tree files
pub(crate) struct ListFilesCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// File pattern
    #[arg(long, value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// File type
    #[arg(long, value_name = "TYPE")]
    pub type_: Option<String>,

    /// Labels for list-files
    #[arg(long)]
    labels: Option<Vec<String>>,
}

impl CliCommand for ListFilesCommand {
    async fn exec(self) -> MgitResult<()> {
        let files = ops::list_files(self.into()).await?;
        println!("{}", files.join("\n"));

        Ok(())
    }
}

impl From<ListFilesCommand> for ListFilesOptions {
    fn from(value: ListFilesCommand) -> Self {
        ListFilesOptions::new(value.path, value.config, value.labels)
    }
}
