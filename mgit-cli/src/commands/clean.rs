use clap::Args;
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, CleanOptions};

use crate::commands::CliCommand;
use crate::term::print_style_message;
use crate::term::progress::MultiProgress;

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
        let progress = MultiProgress::default();
        let msg = ops::clean_repo(self.into(), progress).await?;
        print_style_message(&msg);
        Ok(())
    }
}

impl From<CleanCommand> for CleanOptions {
    fn from(value: CleanCommand) -> Self {
        CleanOptions::new(value.path, value.config, value.labels)
    }
}
