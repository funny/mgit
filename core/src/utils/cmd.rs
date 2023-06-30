use crate::core::repo::RepoId;
use anyhow::{Context, Error};
use console::strip_ansi_codes;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::utils::progress::Progress;
use crate::utils::StyleMessage as SMsg;

pub fn exec_cmd(path: impl AsRef<Path>, cmd: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let mut command = std::process::Command::new(cmd);
    let full_command = command.current_dir(path).args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        full_command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = full_command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| format!("Error starting command: {:?}", full_command))?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    match output.status.success() {
        true => Ok(stdout),
        false => Err(anyhow::anyhow!(stderr)),
    }
}

pub fn exec_cmd_with_progress(
    rel_path: impl AsRef<str>,
    command: &mut Command,
    prefix: &str,
    idx: usize,
    progress: &impl Progress,
) -> anyhow::Result<()> {
    let mut spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| SMsg::new().plain_text(format!("Error starting command {:?}", command)))?;

    let last_line = SMsg::spinner_info(prefix, &rel_path, "running...".into());
    progress.repo_info(RepoId::new(idx, rel_path.as_ref()), last_line);

    // get message from stderr with "--progress" option
    let mut last_line = SMsg::new();
    if let Some(ref mut stderr) = spawned.stderr {
        let lines = BufReader::new(stderr).split(b'\r');
        for line in lines {
            let output = line.unwrap();
            if output.is_empty() {
                continue;
            }
            let line = std::str::from_utf8(&output).unwrap();
            let plain_line = strip_ansi_codes(line).replace('\n', " ");
            let full_line = SMsg::spinner_info(prefix, &rel_path, plain_line.trim().into());

            progress.repo_info(RepoId::new(idx, rel_path.as_ref()), full_line);
            last_line = last_line.plain_text(plain_line);
        }
    }

    let exit_code = spawned
        .wait()
        .context("Error waiting for process to finish")?;

    if !exit_code.success() {
        return Err(Error::msg("").context(
            SMsg::new()
                .plain_text(format!(
                    "Git exited with code {}: ",
                    exit_code.code().unwrap()
                ))
                .join(last_line)
                .plain_text(format!(". With command : {:?}", command)),
        ));
    }
    Ok(())
}
