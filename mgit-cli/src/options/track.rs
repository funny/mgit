use clap::Args;
use mgit::ops::TrackOptions as CoreTrackOptions;
use mgit::options::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Track remote branch
pub(crate) struct TrackOptions {
    #[clap(flatten)]
    base: BaseOptions,

    /// Ignore specified repositories for track
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl From<TrackOptions> for CoreOptions {
    fn from(value: TrackOptions) -> Self {
        CoreOptions::new_track_options(value.base.path, value.base.config, value.ignore)
    }
}
