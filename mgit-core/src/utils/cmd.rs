use snafu::ResultExt;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::error::{GitCommandFailedSnafu, MgitResult, ProcessWaitFailedSnafu};
use crate::utils::process_guard::ProcessGuard;
use crate::utils::progress::{Progress, RepoInfo};

pub async fn exec_cmd(path: impl AsRef<Path>, cmd: &str, args: &[&str]) -> MgitResult<String> {
    let mut command = Command::new(cmd);
    command.current_dir(&path).args(args);

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = command.output().await.context(GitCommandFailedSnafu {
        command: format!("{:?}", command),
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        return Err(crate::error::MgitError::GitCommandError {
            code: output.status.code().unwrap_or(-1),
            output: stderr,
        });
    }
}

pub async fn exec_cmd_with_progress(
    repo_info: &RepoInfo<'_>,
    command: &mut Command,
    progress: &impl Progress,
) -> MgitResult<()> {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = command.spawn().context(GitCommandFailedSnafu {
        command: format!("{:?}", command),
    })?;

    // Attach to Job Object for safety
    ProcessGuard::attach(&child);

    progress.on_repo_update(repo_info, "running...".into());

    let output = child
        .wait_with_output()
        .await
        .context(ProcessWaitFailedSnafu)?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(crate::error::MgitError::GitCommandError {
            code: output.status.code().unwrap_or(-1),
            output: stderr_str,
        });
    } else {
        Ok(())
    }
}

pub async fn retry<T, F, Fut>(times: usize, sleep: std::time::Duration, f: F) -> MgitResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = MgitResult<T>>,
{
    let mut last_err = None;
    for _ in 0..times {
        match f().await {
            Ok(r) => return Ok(r),
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(sleep).await;
            }
        }
    }
    Err(last_err.unwrap())
}
