#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    None,
    Init,
    Snapshot,
    Refresh,
    Fetch,
    Sync,
    SyncHard,
    Track,
    NewBranch,
    NewTag,
    Clean,
}
