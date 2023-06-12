use clap::Args;
use mgit::ops::TrackOptions;
use std::path::PathBuf;

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

impl From<TrackCommand> for TrackOptions {
    fn from(value: TrackCommand) -> Self {
        TrackOptions::new(value.path, value.config, value.ignore)
    }
}
