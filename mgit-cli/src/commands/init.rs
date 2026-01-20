use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, InitOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Init git repos
pub(crate) struct InitCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Force overwrite config file
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// Ignore specified repositories to init
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommand for InitCommand {
    async fn exec(self) -> MgitResult<()> {
        let msg = ops::init_repo(self.into()).await?;
        println!("{}", msg);
        Ok(())
    }
}

impl From<InitCommand> for InitOptions {
    fn from(value: InitCommand) -> Self {
        InitOptions::new(value.path, Some(value.force))
    }
}
