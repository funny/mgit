#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    None,
    Init,
    Snapshot,
    Fetch,
    Sync,
    SyncHard,
    Track,
    Clean,
    Refresh,
}
