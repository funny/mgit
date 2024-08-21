use clap::Args;
use std::path::PathBuf;

use mgit::ops::{self, NewBranchOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// new branch base on current branch in config
pub(crate) struct NewBranchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// new branch name
    #[arg(long, value_name = "NAME")]
    pub name: String,

    #[arg(long, value_name = "FILE")]
    pub new_config: Option<PathBuf>,

    /// Ignore specified repositories to create new branch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for NewBranchCommand {
    fn exec(self) -> MgitResult {
        ops::new_branch(self.into())
    }
}

impl From<NewBranchCommand> for NewBranchOptions {
    fn from(value: NewBranchCommand) -> Self {
        NewBranchOptions::new(
            value.path,
            value.config,
            value.new_config,
            value.name,
            value.ignore,
        )
    }
}
