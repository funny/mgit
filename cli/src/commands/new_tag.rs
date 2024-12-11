use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, NewTagOptions};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// new tag base local branch configed in gitrepos
pub(crate) struct NewTagCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// New tag name
    #[arg(long, value_name = "TAG")]
    pub tag: String,

    /// Push tag to remote
    #[arg(long, action = ArgAction::SetTrue)]
    pub push: bool,

    /// Ignore specified repositories to create new branch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommad for NewTagCommand {
    fn exec(self) -> MgitResult {
        ops::new_tag(self.into())
    }
}

impl From<NewTagCommand> for NewTagOptions {
    fn from(value: NewTagCommand) -> Self {
        NewTagOptions::new(
            value.path,
            value.config,
            value.tag,
            value.push,
            value.ignore,
        )
    }
}
