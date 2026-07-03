pub mod progress;

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use mgit::utils::shell::{Auth, ShellInteraction};
use mgit::utils::style_message::StyleMessage;

static COLOR_OUTPUT_ENABLED: AtomicBool = AtomicBool::new(true);

pub(crate) fn configure_color(enabled: bool) {
    console::set_colors_enabled(enabled);
    console::set_colors_enabled_stderr(enabled);
    COLOR_OUTPUT_ENABLED.store(enabled, Ordering::Relaxed);

    #[cfg(windows)]
    if enabled {
        let _ = ansi_term::enable_ansi_support();
    }
}

pub(crate) fn colors_enabled() -> bool {
    COLOR_OUTPUT_ENABLED.load(Ordering::Relaxed)
}

pub(crate) fn render_style_message(message: &StyleMessage) -> String {
    if colors_enabled() {
        message.to_string()
    } else {
        message.to_plain_text()
    }
}

pub(crate) fn print_style_message(message: &StyleMessage) {
    println!("{}", render_style_message(message));
}

#[derive(Default)]
pub struct TerminalShell;

impl TerminalShell {
    #[allow(dead_code)]
    fn prompt_line(prompt: &str) -> io::Result<String> {
        let mut stderr = io::stderr();
        write!(stderr, "{}", prompt)?;
        stderr.flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}

impl ShellInteraction for TerminalShell {
    fn warn(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "{}", msg);
    }

    fn ask_ssh_trust(&self, fingerprint: &str) -> bool {
        let prompt = format!("Trust SSH fingerprint {}? [y/N]: ", fingerprint);
        match Self::prompt_line(&prompt) {
            Ok(s) => matches!(s.as_str(), "y" | "Y" | "yes" | "YES" | "Yes"),
            Err(_) => false,
        }
    }

    fn ask_http_auth(&self) -> Option<Auth> {
        let username = Self::prompt_line("HTTP username (empty to cancel): ").ok()?;
        if username.is_empty() {
            return None;
        }
        let password = Self::prompt_line("HTTP password: ").ok()?;
        Some(Auth { username, password })
    }
}
