use clap::Args;
use std::path::PathBuf;

use mgit::ops::{self, TrackOptions};
use mgit::utils::error::MgitResult;

use crate::utils::progress::MultiProgress;
use crate::CliCommad;

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

impl CliCommad for TrackCommand {
    fn exec(self) -> MgitResult {
        let progress = MultiProgress::default();
        ops::track(self.into(), progress)
    }
}

impl From<TrackCommand> for TrackOptions {
    fn from(value: TrackCommand) -> Self {
        TrackOptions::new(value.path, value.config, value.ignore)
    }
}
