use clap::{ArgAction, Args};
use std::path::PathBuf;

use mgit::ops::{self, SnapshotOptions, SnapshotType};
use mgit::utils::error::MgitResult;

use crate::CliCommad;

/// Snapshot git repos
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct SnapshotCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// snapshot by branch
    #[arg(long, action = ArgAction::SetTrue)]
    pub branch: bool,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// Ignore specified repositories for snapshot
    #[arg(long)]
    pub ignore: Option<Vec<String>>,
}

impl CliCommad for SnapshotCommand {
    fn exec(self) -> MgitResult {
        ops::snapshot_repo(self.into())
    }
}

impl From<SnapshotCommand> for SnapshotOptions {
    fn from(value: SnapshotCommand) -> Self {
        SnapshotOptions::new(
            value.path,
            value.config,
            Some(value.force),
            match value.branch {
                true => Some(SnapshotType::Branch),
                false => Some(SnapshotType::Commit),
            },
            value.ignore,
        )
    }
}
