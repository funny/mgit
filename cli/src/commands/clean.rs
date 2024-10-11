use clap::Args;
use std::path::PathBuf;

use mgit::ops::{self, CleanOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Clean unused git repos
pub(crate) struct CleanCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

impl CliCommad for CleanCommand {
    fn exec(self) -> MgitResult {
        ops::clean_repo(self.into())
    }
}

impl From<CleanCommand> for CleanOptions {
    fn from(value: CleanCommand) -> Self {
        CleanOptions::new(value.path, value.config)
    }
}
