use std::collections::{HashMap};
use serde::{Deserialize, Serialize};

pub mod init;
pub mod sync;
pub mod fetch;
pub mod clean;

/// this type is used to deserialize `.gitrepos` files.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlConfig {
    version: Option<String>,
    default_branch: Option<String>,
    default_remote: Option<String>,
    repos: Option<HashMap<String, Vec<TomlRepo>>>,
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
