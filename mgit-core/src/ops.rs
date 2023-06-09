mod clean;
mod fetch;
mod init;
mod list_files;
mod snapshot;
mod sync;
mod track;

pub use clean::{clean_repo, CleanOptions};
pub use fetch::{fetch_repos, FetchOptions};
pub use init::{init_repo, InitOptions};
pub use list_files::{list_files, ListFilesOptions};
pub use snapshot::{snapshot_repo, SnapshotOptions};
pub use sync::{sync_repo, SyncOptions};
pub use track::{track, TrackOptions};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StashMode {
    Normal,
    Stash,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResetType {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SnapshotType {
    Commit,
    Branch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RemoteRef {
    Commit(String),
    Tag(String),
    Branch(String),
}

// pub fn builtin_exec(cmd: &str) -> Option<fn(&ArgMatches)> {
//     let f = match cmd {
//         "init" => init::exec,
//         "snapshot" => snapshot::exec,
//         "fetch" => fetch::exec,
//         "sync" => sync::exec,
//         "track" => track::exec,
//         "clean" => clean::exec,
//         "ls-files" => ls_files::exec,
//         _ => return None,
//     };
//     Some(f)
// }
