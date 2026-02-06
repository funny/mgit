pub mod cmd;
pub mod label;
pub mod path;
pub mod process_guard;
pub mod progress;
pub mod shell;
pub mod style_message;

pub use style_message::StyleMessage;

/// Safe wrapper for getting current working directory.
/// Returns the current directory or the provided fallback path if getting cwd fails.
pub fn current_dir_or(fallback: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| fallback.as_ref().to_path_buf())
}

/// Get current working directory with a descriptive error message.
/// Panics only if the system cannot determine the current directory.
#[inline]
pub fn current_dir() -> std::path::PathBuf {
    env::current_dir().expect("Failed to get current working directory")
}

/// Result type alias for directory operations
pub type DirResult<T = std::path::PathBuf> = Result<T, std::io::Error>;

/// Get the current working directory as a Result.
pub fn get_current_dir() -> DirResult {
    env::current_dir()
}

use std::env;
