use std::env;
use std::path::{Path, PathBuf};

use crate::ops::{
    CleanOptions, FetchOptions, InitOptions, ListFilesOptions, SnapshotOptions, SyncOptions,
    TrackOptions,
};
use crate::ops::{SnapshotType, StashMode};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct CoreOptions {
    pub path: PathBuf,

    pub force: Option<bool>,

    pub thread_count: Option<usize>,

    pub silent: Option<bool>,

    pub depth: Option<usize>,

    pub ignore: Option<Vec<String>>,

    pub config_path: Option<PathBuf>,

    pub hard: Option<bool>,

    pub stash: Option<bool>,

    pub no_track: Option<bool>,

    pub no_checkout: Option<bool>,

    pub snapshot_type: Option<SnapshotType>,

    pub stash_mode: Option<StashMode>,
}

impl InitOptions for CoreOptions {
    fn new_init_options(path: Option<impl AsRef<Path>>, force: Option<bool>) -> Self {
        Self {
            path: path
                .map(|p| PathBuf::from(p.as_ref()))
                .map_or(env::current_dir().unwrap(), |p| p),
            force: force.map_or(Some(true), Some),
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn force(&self) -> bool {
        self.force.unwrap()
    }
}

impl SnapshotOptions for CoreOptions {
    fn new_snapshot_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        force: Option<bool>,
        snapshot_type: Option<SnapshotType>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path: Some(config_path),
            force: force.map_or(Some(false), Some),
            snapshot_type: snapshot_type.map_or(Some(SnapshotType::Commit), Some),
            ignore,
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }

    fn force(&self) -> bool {
        self.force.unwrap()
    }

    fn snapshot_type(&self) -> &SnapshotType {
        self.snapshot_type.as_ref().unwrap()
    }

    fn ignore(&self) -> Option<&Vec<String>> {
        self.ignore.as_ref()
    }
}

impl FetchOptions for CoreOptions {
    fn new_fetch_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path: Some(config_path),
            thread_count: thread.map_or(Some(4), Some),
            silent: silent.map_or(Some(false), Some),
            depth,
            ignore,
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }

    fn thread_count(&self) -> usize {
        self.thread_count.unwrap()
    }

    fn silent(&self) -> bool {
        self.silent.unwrap()
    }

    fn depth(&self) -> Option<usize> {
        self.depth
    }

    fn ignore(&self) -> Option<&Vec<String>> {
        self.ignore.as_ref()
    }
}

impl CleanOptions for CoreOptions {
    fn new_clean_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path: Some(config_path),
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }
}

impl ListFilesOptions for CoreOptions {
    fn new_list_files_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
    ) -> Self {
        Self::new_clean_options(path, config_path)
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }
}

impl TrackOptions for CoreOptions {
    fn new_track_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path: Some(config_path),
            ignore,
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }

    fn ignore(&self) -> Option<&Vec<String>> {
        self.ignore.as_ref()
    }
}

impl SyncOptions for CoreOptions {
    fn new_sync_options(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        thread_count: Option<usize>,
        silent: Option<bool>,
        depth: Option<usize>,
        ignore: Option<Vec<String>>,
        hard: Option<bool>,
        stash: Option<bool>,
        no_track: Option<bool>,
        no_checkout: Option<bool>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path: Some(config_path),
            thread_count: thread_count.map_or(Some(4), Some),
            silent: silent.map_or(Some(false), Some),
            depth,
            ignore,
            hard: hard.map_or(Some(false), Some),
            stash: stash.map_or(Some(false), Some),
            no_track: no_track.map_or(Some(false), Some),
            no_checkout: no_checkout.map_or(Some(false), Some),
            ..Default::default()
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn config_path(&self) -> &PathBuf {
        self.config_path.as_ref().unwrap()
    }

    fn thread_count(&self) -> usize {
        self.thread_count.unwrap()
    }

    fn silent(&self) -> bool {
        self.silent.unwrap()
    }

    fn depth(&self) -> Option<usize> {
        self.depth
    }

    fn ignore(&self) -> Option<&Vec<String>> {
        self.ignore.as_ref()
    }

    fn hard(&self) -> bool {
        self.hard.unwrap()
    }

    fn stash(&self) -> bool {
        self.stash.unwrap()
    }

    fn no_track(&self) -> bool {
        self.no_track.unwrap()
    }

    fn no_checkout(&self) -> bool {
        self.no_checkout.unwrap()
    }
}
