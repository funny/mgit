use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod clean;
pub mod fetch;
pub mod init;
pub mod sync;

/// this type is used to deserialize `.gitrepos` files.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlConfig {
    version: Option<String>,
    default_branch: Option<String>,
    default_remote: Option<String>,
    repos: Option<BTreeMap<String, Vec<TomlRepo>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlRepo {
    local: Option<String>,
    remote: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    commit: Option<String>,
}
