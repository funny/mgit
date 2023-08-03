use crate::utils::StyleMessage;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub type MgitResult<T = StyleMessage, E = anyhow::Error> = Result<T, E>;

#[derive(Debug, Error)]
pub enum MgitError {
    #[error("{0}")]
    DirNotFound(StyleMessage),

    #[error("{0}")]
    DirAlreadyInited(StyleMessage),

    #[error("{0}")]
    ConfigFileNotFound(StyleMessage),

    #[error("Load config file failed!")]
    LoadConfigFailed,

    #[error("Create thread pool failed!")]
    CreateThreadPoolFailed,

    #[error("{prefix}\nErrors:\n{errors}")]
    OpsError {
        prefix: StyleMessage,
        errors: OpsErrors,
    },
}

#[derive(Debug)]
pub struct OpsErrors(pub Vec<StyleMessage>);

impl From<Vec<StyleMessage>> for OpsErrors {
    fn from(value: Vec<StyleMessage>) -> Self {
        Self(value)
    }
}

impl Display for OpsErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}
