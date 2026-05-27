use snafu::Snafu;
use std::path::PathBuf;

/// Result type alias for MGIT operations
pub type MgitResult<T = ()> = Result<T, MgitError>;

/// Error types for MGIT operations
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

    // === Specific Operation Errors (replacing generic OpsError) ===
    #[snafu(display("Semaphore acquisition failed: {}", message))]
    AcquirePermitFailed { message: String },

    #[snafu(display("Invalid repository configuration: {}", message))]
    InvalidRepoConfig { message: String },

    #[snafu(display("Repository has no remote configured: {}", path.display()))]
    NoRemoteConfigured { path: PathBuf },

    #[snafu(display("Branch reference required but not found: {}", message))]
    BranchReferenceRequired { message: String },

    #[snafu(display("Failed to create directory {}: {}", path.display(), source))]
    CreateDirFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Stash and hard reset cannot be used together"))]
    StashHardConflict,

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
