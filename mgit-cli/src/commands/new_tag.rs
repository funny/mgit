use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, NewTagOptions};

use crate::commands::CliCommand;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// New tag
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

    /// Ignore specified repositories to create new tag
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommand for NewTagCommand {
    async fn exec(self) -> MgitResult<()> {
        let msg = ops::new_tag(self.into()).await?;
        println!("{}", msg);
        Ok(())
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
