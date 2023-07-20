use serde::{Deserialize, Serialize};

use mgit::core::repo::TomlRepo;

pub(crate) mod project_settings;
pub(crate) mod user_settings;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SyncType {
    Normal,
    Stash,
    Hard,
}

pub fn cmp_toml_repo(dest: &TomlRepo, src: &TomlRepo) -> bool {
    let mut result = false;
    if dest.branch != src.branch || dest.tag != src.tag || dest.commit != src.commit {
        result = true;
    }
    result
}
