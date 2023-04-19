use anyhow::Context;
use console::strip_ansi_codes;
use indicatif::ProgressBar;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

use super::logger;

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
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let mut spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Error starting command {:?}", command))?;

    let mut last_line = logger::fmt_spinner_desc(prefix, &rel_path, "running...");
    progress_bar.set_message(logger::truncate_spinner_msg(&last_line));

    // get message from stderr with "--progress" option
    if let Some(ref mut stderr) = spawned.stderr {
        let lines = BufReader::new(stderr).split(b'\r');
        for line in lines {
            let output = line.unwrap();
            if output.is_empty() {
                continue;
            }
            let line = std::str::from_utf8(&output).unwrap();
            let plain_line = strip_ansi_codes(line).replace('\n', " ");
            let full_line = logger::fmt_spinner_desc(prefix, &rel_path, plain_line.trim());

            progress_bar.set_message(logger::truncate_spinner_msg(&full_line));
            last_line = plain_line;
        }
    }

    let exit_code = spawned
        .wait()
        .context("Error waiting for process to finish")?;

    if !exit_code.success() {
        return Err(anyhow::anyhow!(
            "Git exited with code {}: {}. With command : {:?}.",
            exit_code.code().unwrap(),
            last_line.trim(),
            command
        ));
    }

    Ok(())
}
