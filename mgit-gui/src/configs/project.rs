use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha256::digest;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TomlProjectSettings {
    pub project: Option<String>,
    pub recent_configs: Option<Vec<String>>,
    pub snapshot_ignore: Option<String>,

    pub new_branch_ignore: Option<String>,
    pub new_branch_name: Option<String>,
    pub new_branch_config_path: Option<String>,

    pub new_tag_ignore: Option<String>,
    pub new_tag_name: Option<String>,
    pub new_tag_push: Option<bool>,

    pub ignore: Option<String>,

    #[serde(default)]
    pub labels: BTreeSet<String>,
}

impl TomlProjectSettings {
    pub fn load(project: String, is_hash_name: bool) -> Self {
        let mut recent_config = TomlProjectSettings::default();
        if let Some(path) = home::home_dir() {
            let recent_dir = path.join(".mgit/tmp");
            let recent_file = match is_hash_name {
                true => recent_dir.join(project),
                false => {
                    let hash_name = digest(project);
                    recent_dir.join(hash_name)
                }
            };

            if recent_file.is_file() {
                let txt = std::fs::read_to_string(path.join(recent_file)).unwrap();
                recent_config = toml::from_str(txt.as_str()).unwrap();
            }
        }
        recent_config
    }

    pub fn save(&self, project: String) {
        if let Some(path) = home::home_dir() {
            let recent_dir = path.join(".mgit/tmp");
            let hash_name = digest(project);
            let recent_file = recent_dir.join(hash_name);

            let toml_string = self.serialize();

            let _ = std::fs::create_dir_all(&recent_dir);
            std::fs::write(recent_file, toml_string).expect("Failed to write file");
        }
    }

    pub fn push_recent_config(&mut self, new_path: String) {
        if let Some(history) = &mut self.recent_configs {
            get_new_history(new_path, history);
        } else {
            self.recent_configs = Some(vec![new_path.clone()]);
        }
    }

    pub fn save_ignore(&mut self, path: String, is_ignore: bool) {
        let mut ignore = self.ignore.clone().unwrap_or(String::new());
        let new_ignore = format!("{}\n", &path);

        match (is_ignore, ignore.contains(&new_ignore)) {
            (false, true) => ignore = ignore.replace(&new_ignore, ""),
            (true, false) => ignore.push_str(&new_ignore),
            _ => {}
        };

        self.ignore = match ignore.is_empty() {
            true => None,
            false => Some(ignore),
        }
    }

    pub fn save_snapshot_ignore(&mut self, snapshot_ignore: String) {
        self.snapshot_ignore = match snapshot_ignore.is_empty() {
            true => None,
            false => Some(snapshot_ignore),
        };
    }

    pub fn save_new_branch_ignore(&mut self, new_branch_ignore: String) {
        self.new_branch_ignore = match new_branch_ignore.is_empty() {
            true => None,
            false => Some(new_branch_ignore),
        };
    }

    pub fn save_new_branch_name(&mut self, branch_name: String) {
        self.new_branch_name = match branch_name.is_empty() {
            true => None,
            false => Some(branch_name),
        };
    }

    pub fn save_new_branch_config_path(&mut self, config_path: String) {
        self.new_branch_config_path = match config_path.is_empty() {
            true => None,
            false => Some(config_path),
        };
    }

    pub fn save_new_tag_ignore(&mut self, new_tag_ignore: String) {
        self.new_tag_ignore = match new_tag_ignore.is_empty() {
            true => None,
            false => Some(new_tag_ignore),
        };
    }

    pub fn save_new_tag_name(&mut self, tag_name: String) {
        self.new_tag_name = match tag_name.is_empty() {
            true => None,
            false => Some(tag_name),
        };
    }

    pub fn save_new_tag_push(&mut self, push: bool) {
        self.new_tag_push = Some(push);
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str("# This file is about recent config.\n");

        if let Ok(toml_string) = toml_edit::ser::to_string_pretty(&self) {
            out.push('\n');
            out.push_str(&toml_string)
        }
        out
    }
}

pub(crate) fn get_new_history(path: String, history: &mut Vec<String>) {
    if !history.is_empty() {
        if history.contains(&path) {
            let idx = history.iter().position(|r| *r == path).unwrap();
            history.remove(idx);
        }
        history.insert(0, path);

        if history.len() > 20 {
            history.pop();
        }
    } else {
        *history = vec![path];
    }
}
