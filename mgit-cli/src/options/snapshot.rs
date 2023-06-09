use clap::{ArgAction, Args};
use mgit::ops::{SnapshotOptions as CoreSnapshotOptions, SnapshotType};
use mgit::options::CoreOptions;

use crate::cli::BaseOptions;

/// Snapshot git repos
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct SnapshotOptions {
    #[clap(flatten)]
    pub base: BaseOptions,

    /// snapshot by branch
    #[arg(long, action = ArgAction::SetFalse)]
    pub branch: bool,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// Ignore specified repositories for snapshot
    #[arg(long)]
    pub ignore: Option<Vec<String>>,
}

impl From<SnapshotOptions> for CoreOptions {
    fn from(value: SnapshotOptions) -> Self {
        CoreOptions::new_snapshot_options(
            value.base.path,
            value.base.config,
            Some(value.force),
            match value.branch {
                true => Some(SnapshotType::Branch),
                false => Some(SnapshotType::Commit),
            },
            value.ignore,
        )
    }
}
