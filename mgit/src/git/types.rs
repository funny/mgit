#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StashMode {
    Normal,
    Stash,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResetType {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RemoteRef {
    Commit(String),
    Tag(String),
    Branch(String),
}
