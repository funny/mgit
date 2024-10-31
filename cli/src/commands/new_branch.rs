use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, NewBranchOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// new branch base on current branch in config
pub(crate) struct NewRemoteBranchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// new branch name
    #[arg(long, value_name = "BRANCH")]
    pub branch: String,

    #[arg(long, value_name = "FILE")]
    pub new_config: Option<PathBuf>,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// Ignore specified repositories to create new branch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for NewRemoteBranchCommand {
    fn exec(self) -> MgitResult {
        ops::new_remote_branch(self.into())
    }
}

impl From<NewRemoteBranchCommand> for NewBranchOptions {
    fn from(value: NewRemoteBranchCommand) -> Self {
        NewBranchOptions::new(
            value.path,
            value.config,
            value.new_config,
            value.branch,
            value.force,
            value.ignore,
        )
    }
}
