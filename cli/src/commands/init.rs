use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, InitOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct InitCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,
}

impl CliCommad for InitCommand {
    fn exec(self) -> MgitResult {
        ops::init_repo(self.into())
    }
}

impl From<InitCommand> for InitOptions {
    fn from(value: InitCommand) -> Self {
        InitOptions::new(value.path, Some(value.force))
    }
}
