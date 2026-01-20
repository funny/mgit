use mgit::ops::{NewBranchOptions, NewTagOptions};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    None,
    // Basic
    Init,
    Snapshot,
    // Routine
    Fetch,
    Sync,
    SyncHard,
    Refresh,
    // Changes
    Track,
    Clean,
    // Branch/Tag
    NewBranch,
    NewTag,
}

#[derive(Debug)]
pub(crate) enum Event {
    Input(InputEvent),
    Action(Action),
    Backend(BackendEvent),
}

#[derive(Debug)]
pub(crate) enum InputEvent {
    ProjectPathChanged(String),
    ConfigFileChanged(String),
}

#[derive(Debug)]
pub(crate) enum Action {
    RunOps(OpsCommand),
    RunOpsBatch(Vec<OpsCommand>),
    Refresh,
    RetryConfigSave,
    SaveOptions,
    SaveSnapshotIgnore,
    SaveNewBranchOption,
    SaveNewTagOption,
    ExitApp,
}

#[derive(Debug)]
pub(crate) enum BackendEvent {
    RepoStateUpdated {
        run_id: u64,
        id: usize,
        repo_state: crate::app::context::RepoState,
    },
    CommandFinished {
        run_id: u64,
        command: OpsCommand,
    },
    RemoteBranchesLoaded {
        run_id: u64,
        repo_rel_path: String,
        branches: Vec<String>,
    },
    RemoteBranchesFailed {
        run_id: u64,
        repo_rel_path: String,
        error: String,
    },
    ConfigSaved {
        run_id: u64,
        path: String,
    },
    ConfigSaveFailed {
        run_id: u64,
        path: String,
        content: String,
        error: String,
    },
    ConfigLoadFailed {
        run_id: u64,
        error: String,
    },
}

pub(crate) enum OpsCommand {
    Simple(CommandType),
    Snapshot { config_file: String },
    CreateBranch(NewBranchOptions),
    CreateTag(NewTagOptions),
}

impl std::fmt::Debug for OpsCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple(kind) => write!(f, "Simple({:?})", kind),
            Self::Snapshot { config_file } => f
                .debug_struct("Snapshot")
                .field("config_file", config_file)
                .finish(),
            Self::CreateBranch(_) => write!(f, "CreateBranch(...)"),
            Self::CreateTag(_) => write!(f, "CreateTag(...)"),
        }
    }
}

impl OpsCommand {
    pub(crate) fn kind(&self) -> CommandType {
        match self {
            Self::Simple(kind) => *kind,
            Self::Snapshot { .. } => CommandType::Snapshot,
            Self::CreateBranch(_) => CommandType::NewBranch,
            Self::CreateTag(_) => CommandType::NewTag,
        }
    }
}

impl From<CommandType> for OpsCommand {
    fn from(kind: CommandType) -> Self {
        Self::Simple(kind)
    }
}
