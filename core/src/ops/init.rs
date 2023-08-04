use crate::ops::snapshot::SnapshotType;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use crate::ops::snapshot_repo;
use crate::ops::SnapshotOptions;
use crate::utils::error::MgitResult;

use crate::utils::logger;
use crate::utils::style_message::StyleMessage;

pub struct InitOptions {
    pub path: PathBuf,
    pub force: bool,
}

impl InitOptions {
    pub fn new(path: Option<impl AsRef<Path>>, force: Option<bool>) -> Self {
        Self {
            path: path
                .map(|p| PathBuf::from(p.as_ref()))
                .map_or(env::current_dir().unwrap(), |p| p),
            force: force.unwrap_or(true),
        }
    }
}

pub fn init_repo(options: InitOptions) -> MgitResult {
    let path = &options.path;
    let force = options.force;
    let snapshot_type = SnapshotType::Branch;
    let config_file = path.join(".gitrepos");

    logger::info(StyleMessage::ops_start("init", path));

    snapshot_repo(SnapshotOptions::new(
        Some(path.to_path_buf()),
        Some(config_file),
        Some(force),
        Some(snapshot_type),
        None,
    ))
}
