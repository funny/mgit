use crate::utils::path::PathExtension;
use ansi_term::{Colour, Style};
use lazy_static::lazy_static;
use std::fmt::{Display, Formatter};
use std::path::Path;

lazy_static! {
    pub static ref RED: Style = Style::new().fg(Colour::Red);
    pub static ref GREEN: Style = Style::new().fg(Colour::Green);
    pub static ref BLUE: Style = Style::new().fg(Colour::Blue);
    pub static ref YELLOW: Style = Style::new().fg(Colour::Yellow);
    pub static ref GREY: Style = Style::new().fg(Colour::Fixed(245));
    pub static ref RED_BOLD: Style = Style::new().fg(Colour::Red).bold();
    pub static ref GREEN_BOLD: Style = Style::new().fg(Colour::Green).bold();
    pub static ref BLUE_BOLD: Style = Style::new().fg(Colour::Blue).bold();
    pub static ref PURPLE_BOLD: Style = Style::new().fg(Colour::Purple).bold();
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleText {
    pub content: String,
    pub style: Option<&'static ansi_term::Style>,
}

impl StyleText {
    pub fn to_plain_text(&self) -> &str {
        self.content.as_str()
    }
}

impl Display for StyleText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .style
                .map(|style| style.paint(&self.content).to_string())
                .unwrap_or(self.content.clone()),
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StyleMessage(pub Vec<StyleText>);

impl StyleMessage {
    pub(crate) fn new() -> Self {
        StyleMessage::default()
    }

    pub(crate) fn plain_text(mut self, content: impl AsRef<str>) -> Self {
        self.0.push(StyleText {
            content: content.as_ref().to_string(),
            style: None,
        });
        self
    }

    pub(crate) fn styled_text(
        mut self,
        content: impl AsRef<str>,
        style: &'static ansi_term::Style,
    ) -> Self {
        self.0.push(StyleText {
            content: content.as_ref().to_string(),
            style: Some(style),
        });
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn contains(&self, pattern: impl AsRef<str>) -> bool {
        self.0.iter().any(|st| st.content == pattern.as_ref())
    }

    pub(crate) fn join(mut self, other: StyleMessage) -> Self {
        other.0.into_iter().for_each(|m| self.0.push(m));
        self
    }

    pub(crate) fn try_join(self, other: Option<StyleMessage>) -> Self {
        match other {
            Some(v) => self.join(v),
            None => self,
        }
    }

    pub(crate) fn remove(mut self, pattern: impl AsRef<str>) -> Self {
        self.0.retain(|st| st.content != pattern.as_ref());
        self
    }

    pub fn replace(&mut self, other: StyleMessage) {
        self.0 = other.0;
    }
}

impl StyleMessage {
    pub fn to_plain_text(&self) -> String {
        self.0
            .iter()
            .map(|st| st.to_plain_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

// format message
impl StyleMessage {
    pub(crate) fn dir_not_found(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text("Directory ")
            .styled_text(path.as_ref().to_str().unwrap(), &PURPLE_BOLD)
            .plain_text(" not found!")
    }

    pub(crate) fn dir_already_inited(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .styled_text(path.as_ref().to_str().unwrap(), &PURPLE_BOLD)
            .plain_text(" already inited, try ")
            .styled_text("--force", &PURPLE_BOLD)
            .plain_text(" instead!")
    }

    pub(crate) fn update_config_succ() -> Self {
        StyleMessage::new()
            .styled_text(".gitrepos", &PURPLE_BOLD)
            .plain_text(" update")
    }

    pub(crate) fn config_file_not_found() -> Self {
        StyleMessage::new()
            .styled_text(".gitrepos", &PURPLE_BOLD)
            .plain_text(" not found, try ")
            .styled_text("init", &PURPLE_BOLD)
            .plain_text(" instead!")
    }

    pub(crate) fn remove_file_failed(path: impl AsRef<Path>, err: anyhow::Error) -> Self {
        StyleMessage::new().plain_text(format!(
            "remove {} files error: {}",
            path.display_path(),
            err
        ))
    }

    pub(crate) fn remove_file_succ(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text("  ")
            .styled_text(path.display_path(), &PURPLE_BOLD)
            .plain_text(": removed ")
    }

    pub(crate) fn remove_repo_succ(amount: u32) -> Self {
        let mut msg = StyleMessage::new();
        msg = match amount {
            0 => msg.plain_text("no repository is removed."),
            1 => msg
                .styled_text("1", &GREEN_BOLD)
                .plain_text(" repository is removed."),
            _ => msg
                .styled_text(amount.to_string(), &GREEN_BOLD)
                .plain_text(" repositories are removed."),
        };
        msg
    }

    pub(crate) fn ops_start(ops: impl AsRef<str>, path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text(format!("{} in ", ops.as_ref()))
            .styled_text(path.as_ref().display().to_string(), &PURPLE_BOLD)
    }

    pub(crate) fn ops_errors(prefix: impl AsRef<str>, count: usize) -> Result<Self, Self> {
        match count {
            0 => Ok(StyleMessage::new()
                .plain_text(format!("{} finished! 0 error(s).", prefix.as_ref()))),
            _ => Err(StyleMessage::new()
                .plain_text(format!("{} finished! ", prefix.as_ref()))
                .styled_text(count.to_string(), &RED_BOLD)
                .plain_text(" error(s).")),
        }
    }

    pub(crate) fn repo_end(is_success: bool) -> Self {
        let (sign, style): (&str, &Style) = match is_success {
            true => ("√", &GREEN_BOLD),
            false => ("x", &RED_BOLD),
        };
        StyleMessage::new().styled_text(sign, style)
    }

    pub(crate) fn git_error(rel_path: impl AsRef<str>, error: &anyhow::Error) -> Self {
        let err_msg = error
            .chain()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("");

        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), &PURPLE_BOLD)
            .plain_text(" ")
            .styled_text(err_msg.trim(), &RED)
    }

    pub(crate) fn git_untracked(path: impl AsRef<Path>, desc: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(path.display_path(), &PURPLE_BOLD)
            .plain_text(": ")
            .styled_text(desc.as_ref(), &BLUE)
            .plain_text(" untracked")
    }

    pub(crate) fn git_tracking_succ(
        rel_path: impl AsRef<str>,
        local_branch: impl AsRef<str>,
        remote_desc: impl AsRef<str>,
    ) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), &PURPLE_BOLD)
            .plain_text(": ")
            .styled_text(local_branch.as_ref(), &BLUE)
            .plain_text(" -> ")
            .styled_text(remote_desc.as_ref(), &BLUE)
    }

    pub(crate) fn git_tracking_failed(
        rel_path: impl AsRef<str>,
        remote_desc: impl AsRef<str>,
    ) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), &PURPLE_BOLD)
            .plain_text(": ")
            .styled_text("track failed", &RED)
            .plain_text(" ")
            .styled_text(remote_desc.as_ref(), &BLUE)
            .plain_text(" ")
            .styled_text("not found!", &RED)
    }

    pub(crate) fn git_remote_not_found(remote_ref: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("remote ")
            .styled_text(remote_ref.as_ref(), &BLUE)
            .plain_text(" not found")
    }

    pub(crate) fn git_checking_out(branch: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("checkout ")
            .styled_text(branch.as_ref(), &BLUE)
            .plain_text("...")
    }

    pub(crate) fn git_changes(len: usize) -> Self {
        StyleMessage::new()
            .plain_text(", changes(")
            .styled_text(len.to_string(), &RED)
            .plain_text(")")
    }

    pub(crate) fn git_commits(ahead: impl AsRef<str>, behind: impl AsRef<str>) -> Self {
        let ahead = ahead.as_ref();
        let behind = behind.as_ref();

        let commit_str = match (ahead, behind) {
            ("0", "0") => String::new(),
            (_, "0") => format!("commits({}↑)", ahead),
            ("0", _) => format!("commits({}↓)", behind),
            _ => format!("commits({}↑{}↓)", ahead, behind),
        };

        StyleMessage::new()
            .plain_text(", ")
            .styled_text(commit_str, &YELLOW)
    }

    pub(crate) fn git_unknown_revision() -> Self {
        StyleMessage::new()
            .plain_text(", ")
            .styled_text("unknown revision", &YELLOW)
    }

    pub(crate) fn git_update_to(desc: StyleMessage) -> Self {
        StyleMessage::new()
            .styled_text("update to", &GREEN)
            .plain_text(" ")
            .join(desc)
    }

    pub(crate) fn git_update_to_date(branch_log: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("already update to date.")
            .styled_text(branch_log.as_ref(), &GREY)
    }

    pub(crate) fn git_diff(
        remote_desc: impl AsRef<str>,
        commit_desc: StyleMessage,
        changes_desc: StyleMessage,
    ) -> Self {
        StyleMessage::new()
            .styled_text(remote_desc.as_ref(), &BLUE)
            .join(commit_desc)
            .join(changes_desc)
    }
}

impl Display for StyleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .0
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(""),
        )
    }
}

impl<T> From<T> for StyleMessage
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        StyleMessage::new().plain_text(value.as_ref())
    }
}
