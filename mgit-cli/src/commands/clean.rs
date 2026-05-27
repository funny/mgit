use clap::Args;
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, CleanOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Clean unused git repos
pub(crate) struct CleanCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub labels: Option<Vec<String>>,
}

impl CliCommand for CleanCommand {
    async fn exec(self) -> MgitResult<()> {
        let msg = ops::clean_repo(self.into()).await?;
        println!("{}", msg);
        Ok(())
    }
}

impl From<CleanCommand> for CleanOptions {
    fn from(value: CleanCommand) -> Self {
        CleanOptions::new(value.path, value.config, value.labels)
    }
}
