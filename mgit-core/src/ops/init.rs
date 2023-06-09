use std::path::Path;
use std::path::PathBuf;

use crate::ops::snapshot_repo;
use crate::ops::{SnapshotOptions, SnapshotType};
use crate::options::CoreOptions;

use crate::utils::logger;

pub trait InitOptions {
    fn new_init_options(path: Option<impl AsRef<Path>>, force: Option<bool>) -> Self;
    fn path(&self) -> &PathBuf;
    fn force(&self) -> bool;
}

pub fn init_repo(options: impl InitOptions) {
    let path = options.path();
    let force = options.force();
    let snapshot_type = SnapshotType::Branch;
    let config_file = path.join(".gitrepos");

    logger::command_start("init", path);

    snapshot_repo(CoreOptions::new_snapshot_options(
        Some(path.to_path_buf()),
        Some(config_file),
        Some(force),
        Some(snapshot_type),
        None,
    ))
}
