use std::path::Path;

use crate::error::MgitResult;
use crate::utils::cmd::exec_cmd;

pub async fn get_untrack_files(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = ["ls-files", ".", "--exclude-standard", "--others"];
    exec_cmd(path, "git", &args).await
}

pub async fn get_changed_files(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = ["diff", "--name-only"];
    exec_cmd(path, "git", &args).await
}

pub async fn get_staged_files(path: impl AsRef<Path>) -> MgitResult<String> {
    let args = ["diff", "--cached", "--name-only"];
    exec_cmd(path, "git", &args).await
}

pub async fn get_rev_list_count(
    path: impl AsRef<Path>,
    branch_pair: impl AsRef<str>,
) -> MgitResult<String> {
    let args = ["rev-list", "--count", "--left-right", branch_pair.as_ref()];
    exec_cmd(path, "git", &args).await
}
