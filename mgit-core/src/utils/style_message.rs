use crate::utils::path::PathExtension;
use ansi_term::{Colour, Style};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::OnceLock;

// Lazy static styles using OnceLock for thread safety without lazy_static crate
fn style_red() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Red))
}
fn style_green() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Green))
}
fn style_blue() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Blue))
}
fn style_yellow() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Yellow))
}
fn style_grey() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Fixed(245)))
}
fn style_red_bold() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Red).bold())
}
fn style_green_bold() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Green).bold())
}
#[allow(dead_code)]
fn style_blue_bold() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Blue).bold())
}
fn style_purple_bold() -> &'static Style {
    static STYLE: OnceLock<Style> = OnceLock::new();
    STYLE.get_or_init(|| Style::new().fg(Colour::Purple).bold())
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
                .map(|style: &ansi_term::Style| style.paint(&self.content).to_string())
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

    pub fn to_plain_text(&self) -> String {
        self.0
            .iter()
            .map(|st| st.to_plain_text())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// format message
impl StyleMessage {
    #[allow(dead_code)]
    pub(crate) fn dir_not_found(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text("Directory ")
            .styled_text(path.as_ref().to_str().unwrap(), style_purple_bold())
            .plain_text(" not found!")
    }

    #[allow(dead_code)]
    pub(crate) fn dir_already_inited(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .styled_text(path.as_ref().to_str().unwrap(), style_purple_bold())
            .plain_text(" already inited, try ")
            .styled_text("--force", style_purple_bold())
            .plain_text(" instead!")
    }

    pub(crate) fn update_config_succ() -> Self {
        StyleMessage::new()
            .styled_text(".gitrepos", style_purple_bold())
            .plain_text(" update")
    }

    #[allow(dead_code)]
    pub(crate) fn config_file_not_found() -> Self {
        StyleMessage::new()
            .styled_text(".gitrepos", style_purple_bold())
            .plain_text(" not found, try ")
            .styled_text("init", style_purple_bold())
            .plain_text(" instead!")
    }

    pub(crate) fn remove_file_failed(path: impl AsRef<Path>, err: impl std::fmt::Display) -> Self {
        StyleMessage::new().plain_text(format!(
            "remove {} files error: {}",
            path.display_path(),
            err
        ))
    }

    pub(crate) fn remove_file_succ(path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text("  ")
            .styled_text(path.display_path(), style_purple_bold())
            .plain_text(": removed ")
    }

    pub(crate) fn remove_repo_succ(amount: u32) -> Self {
        let mut msg = StyleMessage::new();
        msg = match amount {
            0 => msg.plain_text("no repository is removed.\n"),
            1 => msg
                .styled_text("1", style_green_bold())
                .plain_text(" repository is removed.\n"),
            _ => msg
                .styled_text(amount.to_string(), style_green_bold())
                .plain_text(" repositories are removed.\n"),
        };
        msg
    }

    pub(crate) fn ops_start(ops: impl AsRef<str>, path: impl AsRef<Path>) -> Self {
        StyleMessage::new()
            .plain_text(format!("{} in ", ops.as_ref()))
            .styled_text(path.as_ref().display().to_string(), style_purple_bold())
    }

    pub(crate) fn ops_success(prefix: impl AsRef<str>) -> Self {
        StyleMessage::new().plain_text(format!("{} finished! 0 error(s).\n", prefix.as_ref()))
    }

    pub(crate) fn ops_failed(prefix: impl AsRef<str>, amount: usize) -> Self {
        StyleMessage::new()
            .plain_text(format!("{} finished! ", prefix.as_ref()))
            .styled_text(amount.to_string(), style_red_bold())
            .plain_text(" error(s).\n")
    }

    pub fn repo_end(is_success: bool) -> Self {
        let (sign, style): (&str, &Style) = match is_success {
            true => ("√", style_green_bold()),
            false => ("x", style_red_bold()),
        };
        StyleMessage::new().styled_text(sign, style)
    }

    pub(crate) fn git_error(rel_path: impl AsRef<str>, error: impl std::fmt::Display) -> Self {
        // Since error is Display, we can just to_string it
        let err_msg = error.to_string();

        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), style_purple_bold())
            .plain_text(" ")
            .styled_text(err_msg.trim(), style_red())
    }

    pub(crate) fn git_error_str(rel_path: impl AsRef<str>, error: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), style_purple_bold())
            .plain_text(" ")
            .styled_text(error.as_ref().trim(), style_red())
    }

    pub(crate) fn git_stash(rel_path: impl AsRef<str>, desc: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), style_purple_bold())
            .plain_text(": ")
            .plain_text(desc.as_ref())
    }

    pub(crate) fn git_untracked(path: impl AsRef<Path>, desc: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(path.display_path(), style_purple_bold())
            .plain_text(": ")
            .styled_text(desc.as_ref(), style_blue())
            .plain_text(" untracked")
    }

    pub(crate) fn git_tracking_succ(
        rel_path: impl AsRef<str>,
        local_branch: impl AsRef<str>,
        remote_desc: impl AsRef<str>,
    ) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), style_purple_bold())
            .plain_text(": ")
            .styled_text(local_branch.as_ref(), style_blue())
            .plain_text(" -> ")
            .styled_text(remote_desc.as_ref(), style_blue())
    }

    pub(crate) fn git_tracking_failed(
        rel_path: impl AsRef<str>,
        remote_desc: impl AsRef<str>,
    ) -> Self {
        StyleMessage::new()
            .styled_text(rel_path.as_ref().display_path(), style_purple_bold())
            .plain_text(": ")
            .styled_text("track failed", style_red())
            .plain_text(" ")
            .styled_text(remote_desc.as_ref(), style_blue())
            .plain_text(" ")
            .styled_text("not found!", style_red())
    }

    pub(crate) fn git_remote_not_found(remote_ref: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("remote ")
            .styled_text(remote_ref.as_ref(), style_blue())
            .plain_text(" not found")
    }

    pub(crate) fn git_checking_out(branch: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("checkout ")
            .styled_text(branch.as_ref(), style_blue())
            .plain_text("...")
    }

    pub(crate) fn git_changes(len: usize) -> Option<Self> {
        match len {
            0 => None,
            _ => Some(StyleMessage::new().styled_text(format!("changes({})", len), style_red())),
        }
    }

    pub(crate) fn git_commits(ahead: impl AsRef<str>, behind: impl AsRef<str>) -> Option<Self> {
        let ahead = ahead.as_ref();
        let behind = behind.as_ref();

        let commit_str = match (ahead, behind) {
            ("0", "0") => String::new(),
            (_, "0") => format!("commits({}↑)", ahead),
            ("0", _) => format!("commits({}↓)", behind),
            _ => format!("commits({}↑{}↓)", ahead, behind),
        };

        match commit_str.is_empty() {
            true => None,
            false => Some(StyleMessage::new().styled_text(commit_str, style_yellow())),
        }
    }

    pub(crate) fn git_unknown_revision() -> Self {
        StyleMessage::new().styled_text("unknown revision", style_yellow())
    }

    pub(crate) fn git_update_to(desc: StyleMessage) -> Self {
        StyleMessage::new()
            .styled_text("update to", style_green())
            .plain_text(" ")
            .join(desc)
    }

    pub(crate) fn git_update_to_date(branch_log: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .plain_text("already update to date.")
            .styled_text(branch_log.as_ref(), style_grey())
    }

    pub(crate) fn git_diff(
        remote_desc: impl AsRef<str>,
        commit_desc: Option<StyleMessage>,
        changes_desc: Option<StyleMessage>,
    ) -> Self {
        let mut diff = StyleMessage::new().styled_text(remote_desc.as_ref(), style_blue());
        if let Some(commit_desc) = commit_desc {
            diff = diff.plain_text(", ").join(commit_desc);
        }
        if let Some(changes_desc) = changes_desc {
            diff = diff.plain_text(", ").join(changes_desc)
        }
        diff
    }

    pub(crate) fn git_new_branch(path: impl AsRef<Path>, branch: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(path.display_path(), style_purple_bold())
            .plain_text(": ")
            .styled_text(branch.as_ref(), style_blue())
    }

    pub(crate) fn git_del_branch(path: impl AsRef<Path>, branch: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(path.display_path(), style_purple_bold())
            .plain_text(": deleted ")
            .styled_text(branch.as_ref(), style_blue())
    }

    pub(crate) fn git_new_tag(path: impl AsRef<Path>, tag: impl AsRef<str>) -> Self {
        StyleMessage::new()
            .styled_text(path.display_path(), style_purple_bold())
            .plain_text(": ")
            .styled_text(tag.as_ref(), style_blue())
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
