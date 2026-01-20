use clap::Args;
use std::path::PathBuf;

use mgit::error::MgitResult;
use mgit::ops::{self, TrackOptions};

use crate::commands::CliCommand;
use crate::term::progress::MultiProgress;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Track remote branch
pub(crate) struct TrackCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Ignore specified repositories for track
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl CliCommand for TrackCommand {
    async fn exec(self) -> MgitResult<()> {
        let progress = MultiProgress::default();
        let msg = ops::track(self.into(), progress).await?;
        println!("{}", msg);
        Ok(())
    }
}

impl From<TrackCommand> for TrackOptions {
    fn from(value: TrackCommand) -> Self {
        TrackOptions::new(value.path, value.config, value.ignore)
    }
}
