use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::core::repo::TomlRepo;

/// this type is used to deserialize `.gitrepos` files.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TomlConfig {
    pub version: Option<String>,
    pub default_branch: Option<String>,
    pub default_remote: Option<String>,
    pub repos: Option<Vec<TomlRepo>>,
}

impl TomlConfig {
    // serialize config file .gitrepos
    pub fn serialize(&self) -> String {
        let toml = toml_edit::ser::to_item(self).unwrap();
        let mut out = String::new();

        out.push_str("# This file is automatically @generated by mgit.\n");
        out.push_str("# Editing it as you wish.\n");

        // version = "x.y.z"
        if let Some(item) = toml.get("version") {
            out.push_str(&format!("version = {}\n", item));
        }

        // default-branch = "your_branch"
        if let Some(item) = toml.get("default-branch") {
            out.push_str(&format!("default-branch = {}\n", item));
        }

        // default-remote = "your_remote"
        if let Some(item) = toml.get("default-remote") {
            out.push_str(&format!("default-remote = {}\n", item));
        }

        out.push('\n');

        // [[repos]]
        if let Some(repos) = toml.get("repos") {
            let list = repos.as_array().expect("repos must be an array");

            for entry in list {
                out.push_str("[[repos]]\n");
                let table = entry.as_inline_table().expect("repo must be table");

                // local = "your/local/path"
                if let Some(item) = table.get("local") {
                    out.push_str(&format!("local = {}\n", item));
                }

                // remote = "your://remote/url"
                if let Some(item) = table.get("remote") {
                    out.push_str(&format!("remote = {}\n", item));
                }

                // branch = "your_branch"
                if let Some(item) = table.get("branch") {
                    out.push_str(&format!("branch = {}\n", item));
                }

                // tag = "your_tag"
                if let Some(item) = table.get("tag") {
                    out.push_str(&format!("tag = {}\n", item));
                }

                // commit = "your_tag"
                if let Some(item) = table.get("commit") {
                    out.push_str(&format!("commit = {}\n", item));
                }

                out.push('\n');
            }
        }

        out
    }
}

/// deserialize config file (.gitrepos) with full file path
pub fn load_config(config_file: impl AsRef<Path>) -> Option<TomlConfig> {
    let mut toml_config = None;
    if config_file.as_ref().is_file() {
        // NOTE: mac not recognize "."
        let txt = fs::read_to_string(config_file)
            .unwrap()
            .replace("\".\"", "\"\"");

        if let Ok(res) = toml::from_str(txt.as_str()) {
            toml_config = Some(res);
        }
    }
    toml_config
}
