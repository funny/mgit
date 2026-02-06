use crate::ops::snapshot::SnapshotType;
use std::path::Path;
use std::path::PathBuf;

use crate::error::MgitResult;
use crate::ops::snapshot_repo;
use crate::ops::SnapshotOptions;
use crate::utils::current_dir;
use crate::utils::style_message::StyleMessage;

pub struct InitOptions {
    pub path: PathBuf,
    pub force: bool,
}

impl InitOptions {
    pub fn new(path: Option<impl AsRef<Path>>, force: Option<bool>) -> Self {
        let path = match path {
            Some(p) => PathBuf::from(p.as_ref()),
            None => current_dir(),
        };
        Self {
            path,
            force: force.unwrap_or(true),
        }
    }
}

pub async fn init_repo(options: InitOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let force = options.force;
    let snapshot_type = SnapshotType::Branch;
    let config_file = path.join(".gitrepos");

    tracing::info!(message = %StyleMessage::ops_start("init", path));

    snapshot_repo(SnapshotOptions::new(
        Some(path.to_path_buf()),
        Some(config_file),
        Some(force),
        Some(snapshot_type),
        None,
    ))
    .await
}
