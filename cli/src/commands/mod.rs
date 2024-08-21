use mgit::utils::error::MgitResult;

pub(crate) use clean::CleanCommand;
pub(crate) use fetch::FetchCommand;
pub(crate) use init::InitCommand;
pub(crate) use list_files::ListFilesCommand;
pub(crate) use log_repos::LogReposCommand;
pub(crate) use new_branch::NewBranchCommand;
pub(crate) use snapshot::SnapshotCommand;
pub(crate) use sync::SyncCommand;
pub(crate) use track::TrackCommand;

mod clean;
mod fetch;
mod init;
mod list_files;
mod log_repos;
mod new_branch;
mod snapshot;
mod sync;
mod track;

pub trait CliCommad {
    fn exec(self) -> MgitResult;
}
