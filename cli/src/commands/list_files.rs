use clap::Args;
use std::path::PathBuf;

use mgit::ops::{self, ListFilesOptions};
use mgit::utils::error::MgitResult;
use mgit::utils::StyleMessage;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// List tree
pub(crate) struct ListFilesCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Labels for list-files
    #[arg(long)]
    labels: Option<Vec<String>>,
}

impl CliCommad for ListFilesCommand {
    fn exec(self) -> MgitResult {
        let files = ops::list_files(self.into())?;
        println!("{}", files.join("\n"));

        Ok(StyleMessage::default())
    }
}

impl From<ListFilesCommand> for ListFilesOptions {
    fn from(value: ListFilesCommand) -> Self {
        ListFilesOptions::new(value.path, value.config, value.labels)
    }
}
