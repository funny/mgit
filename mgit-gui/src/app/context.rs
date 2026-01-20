use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;

use crate::app::events::Event;
use crate::app::repo_manager::RepoManager;
use crate::app::session_manager::SessionManager;

// Re-export types for backward compatibility or convenience
pub use crate::app::repo_manager::{PendingConfigSave, RepoState, StateType};

pub struct AppContext {
    pub event_tx: Sender<Event>,
    pub repo_manager: RepoManager,
    pub session_manager: SessionManager,
    pub(crate) run_id_seq: AtomicU64,
}

impl AppContext {
    pub(crate) fn next_run_id(&self) -> u64 {
        self.run_id_seq.fetch_add(1, Ordering::Relaxed) + 1
    }
}
