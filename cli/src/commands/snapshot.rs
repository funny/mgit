use clap::{ArgAction, Args};
use mgit::ops::{SnapshotOptions, SnapshotType};
use std::path::PathBuf;

/// Snapshot git repos
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct SnapshotCommand {
    /// The work directory
    pub path: Option<PathBuf>,

    /// Use specified config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

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
