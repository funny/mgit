use serde::{Deserialize, Serialize};

pub mod project;
pub mod user;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SyncType {
    Normal,
    Stash,
    Hard,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GuiOptions {
    pub init_force: bool,
    pub snapshot_force: bool,
    pub snapshot_branch: bool,
    pub sync_type: SyncType,
    pub sync_no_checkout: bool,
    pub sync_no_track: bool,
    pub sync_thread: u32,
    pub sync_depth: u32,
    pub fetch_thread: u32,
    pub fetch_depth: u32,
}
