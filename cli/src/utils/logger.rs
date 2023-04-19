use owo_colors::OwoColorize;
use std::path::Path;

use crate::utils::path::display_path;

pub fn get_terminal_width() -> usize {
    match term_size::dimensions() {
        Some((width, _)) => width - 10,
        _ => 70,
    }
}

pub fn new(str: impl AsRef<str>) {
    println!("{}", str.as_ref());
}

pub fn dir_not_found(path: impl AsRef<Path>) {
    println!(
        "Directory {} not found!",
        path.as_ref().display().bold().magenta()
    );
}

pub fn dir_already_inited(path: impl AsRef<Path>) {
    println!(
        "{} already inited, try {} instead!",
        path.as_ref().display(),
        "--force".bold().magenta()
    );
}

pub fn update_config_succ() {
    println!("{} update", ".gitrepos".bold().magenta());
}

pub fn config_file_not_found() {
    println!(
        "{} not found, try {} instead!",
        ".gitrepos".bold().magenta(),
        "init".bold().magenta()
    );
}

pub fn remvoe_file_failed(path: impl AsRef<Path>, err: anyhow::Error) {
    println!(
        "remove {} files error: {}",
        display_path(path.as_ref().to_str().unwrap()),
        err.to_string()
    )
}

pub fn remvoe_file_succ(path: impl AsRef<Path>) {
    println!(
        "  {}: removed ",
        display_path(path.as_ref().to_str().unwrap())
            .bold()
            .magenta()
    );
}

pub fn remove_none_repo_succ() {
    println!("no repository is removed.\n");
}

pub fn remove_one_repo_succ() {
    println!(
        "{} repository is removed.\n",
        "1".to_string().bold().green()
    );
}

pub fn remove_multi_repos_succ(count: u32) {
    println!(
        "{} repositories are removed.\n",
        count.to_string().bold().green()
    );
}

pub fn command_start(command: impl AsRef<str>, path: impl AsRef<Path>) {
    println!(
        "{} in {}",
        command.as_ref(),
        path.as_ref().display().bold().magenta()
    );
}

pub fn error_statistics(prefix: impl AsRef<str>, count: usize) {
    match count {
        0 => println!("{} finished! 0 error(s).", prefix.as_ref()),
        _ => println!(
            "{} finished! {} error(s).",
            prefix.as_ref(),
            count.to_string().bold().red()
        ),
    }
    println!("");
}

pub fn error_detail(rel_path: impl AsRef<str>, error: &anyhow::Error) {
    let mut err_msg = String::new();
    for e in error.chain() {
        err_msg += &e.to_string();
    }

    eprintln!(
        "{} {}",
        display_path(&rel_path).bold().magenta(),
        err_msg.trim().red()
    );
    println!("");
}

pub fn fmt_untrack_desc(path: impl AsRef<Path>, desc: impl AsRef<str>) -> String {
    format!(
        "{}: {} untracked",
        display_path(path.as_ref().to_str().unwrap())
            .bold()
            .magenta(),
        desc.as_ref().blue(),
    )
}

pub fn truncate_spinner_msg(message: impl AsRef<str>) -> String {
    let max_width = get_terminal_width();
    console::truncate_str(message.as_ref(), max_width, "...").to_string()
}

pub fn fmt_spinner_start(prefix: impl AsRef<str>, desc: impl AsRef<str>) -> String {
    format!("{:>9} {}", prefix.as_ref(), desc.as_ref())
}

pub fn fmt_spinner_desc(
    prefix: impl AsRef<str>,
    rel_path: impl AsRef<str>,
    desc: impl AsRef<str>,
) -> String {
    format!(
        "{:>9} {}: {}",
        prefix.as_ref(),
        display_path(&rel_path).bold().magenta(),
        desc.as_ref()
    )
}

pub fn fmt_spinner_finished_prefix(
    prefix: impl AsRef<str>,
    rel_path: impl AsRef<str>,
    is_succ: bool,
) -> String {
    let sign = match is_succ {
        true => "√".bold().green().to_string(),
        false => "x".bold().red().to_string(),
    };

    format!(
        "{} {} {}",
        sign.bold().green(),
        prefix.as_ref(),
        display_path(rel_path.as_ref()).bold().magenta()
    )
}

pub fn fmt_tracking_succ_desc(
    rel_path: impl AsRef<str>,
    local_branch: impl AsRef<str>,
    remote_desc: impl AsRef<str>,
) -> String {
    format!(
        "{}: {} -> {}",
        display_path(rel_path).bold().magenta(),
        local_branch.as_ref().blue(),
        remote_desc.as_ref().blue()
    )
}

pub fn fmt_tracking_failed_desc(rel_path: impl AsRef<str>, remote_desc: impl AsRef<str>) -> String {
    format!(
        "{}: {} {} {}",
        display_path(rel_path).bold().magenta(),
        "track failed,".red(),
        remote_desc.as_ref().blue(),
        "not found!".red()
    )
}

pub fn fmt_remote_not_found(remote_ref: impl AsRef<str>) -> String {
    format!("remote {} not found.", remote_ref.as_ref().blue())
}

pub fn fmt_checkouting(branch: impl AsRef<str>) -> String {
    format!("checkout {}...", branch.as_ref().blue())
}

pub fn fmt_changes_desc(len: usize) -> String {
    format!(", changes({})", len).red().to_string()
}

pub fn fmt_commit_desc(ahead: impl AsRef<str>, behind: impl AsRef<str>) -> String {
    let ahead = ahead.as_ref();
    let behind = behind.as_ref();

    match (ahead, behind) {
        ("0", "0") => String::new(),
        (_, "0") => String::from(", ") + &format!("commits({}↑)", ahead).yellow().to_string(),
        ("0", _) => String::from(", ") + &format!("commits({}↓)", behind).yellow().to_string(),
        _ => {
            String::from(", ")
                + &format!("commits({}↑{}↓)", ahead, behind)
                    .yellow()
                    .to_string()
        }
    }
}

pub fn fmt_unknown_revision_desc() -> String {
    format!(", {}", "unknown revision".yellow())
}

pub fn fmt_update_to_desc(desc: impl AsRef<str>) -> String {
    format!("{} {}", "update to".green(), desc.as_ref())
}

pub fn fmt_update_to_date_desc(branch_log: impl AsRef<str>) -> String {
    format!(
        "already update to date. {}",
        branch_log.as_ref().bright_black()
    )
}

pub fn fmt_diff_desc(
    remote_desc: impl AsRef<str>,
    commit_desc: impl AsRef<str>,
    changes_desc: impl AsRef<str>,
) -> String {
    format!(
        "{}{}{}",
        remote_desc.as_ref().blue(),
        commit_desc.as_ref(),
        changes_desc.as_ref(),
    )
}
