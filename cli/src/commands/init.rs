use clap::{ArgAction, Args};
use mgit::ops::InitOptions;
use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct InitCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,
}

impl From<InitCommand> for InitOptions {
    fn from(value: InitCommand) -> Self {
        InitOptions::new(value.path, Some(value.force))
    }
}
