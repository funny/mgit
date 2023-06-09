use console::Style;
use log::{error, info};
use std::path::Path;

use crate::utils::path::display_path;

pub fn get_terminal_width() -> usize {
    match term_size::dimensions() {
        Some((width, _)) => width - 10,
        _ => 70,
    }
}

pub fn new(str: impl AsRef<str>) {
    info!("{}", str.as_ref());
}

pub fn dir_not_found(path: impl AsRef<Path>) {
    let magenta_bold = Style::new().magenta().bold();
    info!(
        "Directory {} not found!",
        magenta_bold.apply_to(path.as_ref().display())
    );
}

pub fn dir_already_inited(path: impl AsRef<Path>) {
    let magenta_bold = Style::new().magenta().bold();
    info!(
        "{} already inited, try {} instead!",
        path.as_ref().display(),
        magenta_bold.apply_to("--force")
    );
}

pub fn update_config_succ() {
    let magenta_bold = Style::new().magenta().bold();
    info!("{} update", magenta_bold.apply_to(".gitrepos"));
}

pub fn config_file_not_found() {
    let magenta_bold = Style::new().magenta().bold();
    info!(
        "{} not found, try {} instead!",
        magenta_bold.apply_to(".gitrepos"),
        magenta_bold.apply_to("init")
    );
}

pub fn remvoe_file_failed(path: impl AsRef<Path>, err: anyhow::Error) {
    info!(
        "remove {} files error: {}",
        display_path(path.as_ref().to_str().unwrap()),
        err.to_string()
    )
}

pub fn remvoe_file_succ(path: impl AsRef<Path>) {
    let magenta_bold = Style::new().magenta().bold();
    info!(
        "  {}: removed ",
        magenta_bold.apply_to(display_path(path.as_ref().to_str().unwrap()))
    );
}

pub fn remove_none_repo_succ() {
    println!("no repository is removed.\n");
}

pub fn remove_one_repo_succ() {
    let green_bold = Style::new().green().bold();
    info!("{} repository is removed.\n", green_bold.apply_to("1"));
}

pub fn remove_multi_repos_succ(count: u32) {
    let green_bold = Style::new().green().bold();
    info!(
        "{} repositories are removed.\n",
        green_bold.apply_to(count.to_string())
    );
}

pub fn command_start(command: impl AsRef<str>, path: impl AsRef<Path>) {
    let magenta_bold = Style::new().magenta().bold();
    info!(
        "{} in {}",
        command.as_ref(),
        magenta_bold.apply_to(path.as_ref().display())
    );
}

pub fn error_statistics(prefix: impl AsRef<str>, count: usize) {
    let red_bold = Style::new().red().bold();
    match count {
        0 => info!("{} finished! 0 error(s).", prefix.as_ref()),
        _ => info!(
            "{} finished! {} error(s).",
            prefix.as_ref(),
            red_bold.apply_to(count.to_string())
        ),
    }
    info!("");
}

pub fn error_detail(rel_path: impl AsRef<str>, error: &anyhow::Error) {
    let red = Style::new().red();
    let magenta_bold = Style::new().magenta().bold();
    let mut err_msg = String::new();

    for e in error.chain() {
        err_msg += &e.to_string();
    }

    error!(
        "{} {}",
        magenta_bold.apply_to(display_path(&rel_path)),
        red.apply_to(err_msg.trim())
    );
    info!("");
}

pub fn fmt_untrack_desc(path: impl AsRef<Path>, desc: impl AsRef<str>) -> String {
    let blue = Style::new().blue();
    let magenta_bold = Style::new().magenta().bold();

    format!(
        "{}: {} untracked",
        magenta_bold.apply_to(display_path(path.as_ref().to_str().unwrap())),
        blue.apply_to(desc.as_ref()),
    )
}

pub fn truncate_spinner_msg(msg: impl AsRef<str>) -> String {
    let max_width = get_terminal_width();
    console::truncate_str(msg.as_ref(), max_width, "...").to_string()
}

pub fn fmt_spinner_start(prefix: impl AsRef<str>, desc: impl AsRef<str>) -> String {
    format!("{:>9} {}", prefix.as_ref(), desc.as_ref())
}

pub fn fmt_spinner_desc(
    prefix: impl AsRef<str>,
    rel_path: impl AsRef<str>,
    desc: impl AsRef<str>,
) -> String {
    let magenta_bold = Style::new().magenta().bold();
    format!(
        "{:>9} {}: {}",
        prefix.as_ref(),
        magenta_bold.apply_to(display_path(&rel_path)),
        desc.as_ref()
    )
}

pub fn fmt_spinner_finished_prefix(
    prefix: impl AsRef<str>,
    rel_path: impl AsRef<str>,
    is_succ: bool,
) -> String {
    let green_bold = Style::new().green().bold();
    let red_bold = Style::new().red().bold();
    let magenta_bold = Style::new().magenta().bold();

    let sign = match is_succ {
        true => green_bold.apply_to("√"),
        false => red_bold.apply_to("x"),
    };

    format!(
        "{} {} {}",
        sign,
        prefix.as_ref(),
        magenta_bold.apply_to(display_path(rel_path.as_ref()))
    )
}

pub fn fmt_tracking_succ_desc(
    rel_path: impl AsRef<str>,
    local_branch: impl AsRef<str>,
    remote_desc: impl AsRef<str>,
) -> String {
    let blue = Style::new().blue();
    let magenta_bold = Style::new().magenta().bold();

    format!(
        "{}: {} -> {}",
        magenta_bold.apply_to(display_path(rel_path)),
        blue.apply_to(local_branch.as_ref()),
        blue.apply_to(remote_desc.as_ref())
    )
}

pub fn fmt_tracking_failed_desc(rel_path: impl AsRef<str>, remote_desc: impl AsRef<str>) -> String {
    let blue = Style::new().blue();
    let red = Style::new().red();
    let magenta_bold = Style::new().magenta().bold();

    format!(
        "{}: {} {} {}",
        magenta_bold.apply_to(display_path(rel_path)),
        red.apply_to("track failed,"),
        blue.apply_to(remote_desc.as_ref()),
        red.apply_to("not found!")
    )
}

pub fn fmt_remote_not_found(remote_ref: impl AsRef<str>) -> String {
    let blue = Style::new().blue();
    format!("remote {} not found.", blue.apply_to(remote_ref.as_ref()))
}

pub fn fmt_checkouting(branch: impl AsRef<str>) -> String {
    let blue = Style::new().blue();
    format!("checkout {}...", blue.apply_to(branch.as_ref()))
}

pub fn fmt_changes_desc(len: usize) -> String {
    let red = Style::new().red();
    red.apply_to(format!(", changes({})", len)).to_string()
}

pub fn fmt_commit_desc(ahead: impl AsRef<str>, behind: impl AsRef<str>) -> String {
    let ahead = ahead.as_ref();
    let behind = behind.as_ref();
    let yellow = Style::new().yellow();

    let commit_str = match (ahead, behind) {
        ("0", "0") => String::new(),
        (_, "0") => format!("commits({}↑)", ahead),
        ("0", _) => format!("commits({}↓)", behind),
        _ => format!("commits({}↑{}↓)", ahead, behind),
    };

    String::from(", ") + &yellow.apply_to(commit_str).to_string()
}

pub fn fmt_unknown_revision_desc() -> String {
    let yellow = Style::new().yellow();
    format!(", {}", yellow.apply_to("unknown revision"))
}

pub fn fmt_update_to_desc(desc: impl AsRef<str>) -> String {
    let green = Style::new().green();
    format!("{} {}", green.apply_to("update to"), desc.as_ref())
}

pub fn fmt_update_to_date_desc(branch_log: impl AsRef<str>) -> String {
    let gray = Style::new().color256(245);
    format!(
        "already update to date. {}",
        gray.apply_to(branch_log.as_ref())
    )
}

pub fn fmt_diff_desc(
    remote_desc: impl AsRef<str>,
    commit_desc: impl AsRef<str>,
    changes_desc: impl AsRef<str>,
) -> String {
    let blue = Style::new().blue();
    format!(
        "{}{}{}",
        blue.apply_to(remote_desc.as_ref()),
        commit_desc.as_ref(),
        changes_desc.as_ref(),
    )
}
