use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, DelBranchOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Delete remote branch
pub(crate) struct DelRemoteBranchCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Remote branch name
    #[arg(long, value_name = "BRANCH")]
    pub branch: String,

    /// Force remove git repo remote branch without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// Ignore specified repositories
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommand for DelRemoteBranchCommand {
    async fn exec(self) -> MgitResult<()> {
        let msg = ops::del_remote_branch(self.into()).await?;
        println!("{}", msg);
        Ok(())
    }
}

impl From<DelRemoteBranchCommand> for DelBranchOptions {
    fn from(value: DelRemoteBranchCommand) -> Self {
        DelBranchOptions::new(value.path, value.config, value.branch, value.ignore)
    }
}
