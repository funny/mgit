use clap::{ Args};
use std::path::PathBuf;

use mgit::ops::{self, DelBranchOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// new branch base on current branch in config
pub(crate) struct DelRemoteBranchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// new branch name
    #[arg(long, value_name = "BRANCH")]
    pub branch: String,

    /// Ignore specified repositories to create new branch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for DelRemoteBranchCommand {
    fn exec(self) -> MgitResult {
        ops::del_remote_branch(self.into())
    }
}

impl From<DelRemoteBranchCommand> for DelBranchOptions {
    fn from(value: DelRemoteBranchCommand) -> Self {
        DelBranchOptions::new(
            value.path,
            value.config,
            value.branch,
            value.ignore,
        )
    }
}
