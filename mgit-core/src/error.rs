use snafu::Snafu;
use std::path::PathBuf;

pub type MgitResult<T = ()> = Result<T, MgitError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[snafu(context(suffix(Snafu)))] // Explicitly set the suffix
pub enum MgitError {
    #[snafu(display("Directory not found: {}", path.display()))]
    DirNotFound { path: PathBuf },

    #[snafu(display("Directory already initialized: {}", path.display()))]
    DirAlreadyInited { path: PathBuf },

    #[snafu(display("Config file not found: {}", path.display()))]
    ConfigFileNotFound { path: PathBuf },

    #[snafu(display("Failed to load config file: {}", source))]
    LoadConfigFailed { source: std::io::Error },

    #[snafu(display("Failed to parse config file: {}", source))]
    ParseConfigFailed { source: toml::de::Error },

    #[snafu(display("IO error: {}", source))]
    IoError { source: std::io::Error },

    #[snafu(display("Git command failed: {}", command))]
    GitCommandFailed {
        command: String,
        #[snafu(source(from(std::io::Error, Box::new)))]
        source: Box<std::io::Error>,
    },

    #[snafu(display("Git command exited with error code {}: {}", code, output))]
    GitCommandError { code: i32, output: String },

    #[snafu(display("Failed to wait for process: {}", source))]
    ProcessWaitFailed { source: std::io::Error },

    #[snafu(display("Operation failed: {}", message))]
    OpsError { message: String },

    #[snafu(display("Remote reference {} not found", remote_ref))]
    RemoteRefNotFound { remote_ref: String },

    #[snafu(display("Current commit not found"))]
    CommitNotFound,

    #[snafu(display("Current branch not found"))]
    BranchNotFound,

    #[snafu(display("Untracked branch"))]
    Untracked,
}
