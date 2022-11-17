use serde::{Deserialize, Serialize};
use toml;

use super::options_window::OptionsWindow;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SyncType {
    Normal,
    Stash,
    Hard,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TomlSetting {
    // recent
    recent_projects: Option<Vec<String>>,
    recent_configs: Option<Vec<String>>,

    // options
    pub init_force: Option<bool>,
    pub snapshot_force: Option<bool>,
    pub snapshot_branch: Option<bool>,
    pub snapshot_ignore: Option<String>,
    pub sync_type: Option<SyncType>,
    pub sync_no_checkout: Option<bool>,
    pub sync_no_track: Option<bool>,
    pub sync_thread: Option<u32>,
    pub fetch_thread: Option<u32>,

    // --ignore for fetch, sync and track
    pub common_ignore: Option<String>,
}

impl Default for TomlSetting {
    fn default() -> Self {
        Self {
            // recent
            recent_projects: None,
            recent_configs: None,

            // options
            init_force: Some(true),
            snapshot_force: Some(true),
            snapshot_branch: Some(true),
            snapshot_ignore: None,
            sync_type: Some(SyncType::Stash),
            sync_no_checkout: Some(false),
            sync_no_track: Some(false),
            sync_thread: Some(4),
            common_ignore: None,
            fetch_thread: Some(4),
        }
    }
}

impl TomlSetting {
    /// load settings from toml file
    /// create new one if toml file doesn't exsit
    pub fn load_settings() -> TomlSetting {
        if let Some(path) = home::home_dir() {
            let setting_file = path.join(".mgit/settings.toml");
            if setting_file.is_file() {
                let txt = std::fs::read_to_string(setting_file).unwrap();
                let settings: TomlSetting = toml::from_str(txt.as_str()).unwrap();
                return settings;
            }
        }
        let settings = TomlSetting::default();
        settings.save_settings();
        settings
    }

    /// save settings into toml file
    pub fn save_settings(&self) {
        if let Some(path) = home::home_dir() {
            let setting_file = path.join(".mgit/settings.toml");

            let toml_string = self.serialize();

            let _ = std::fs::create_dir(path.join(".mgit"));
            std::fs::write(setting_file, toml_string).expect("Failed to write file settings.toml!");
        }
    }

    /// save_option from options window
    pub fn save_options(&mut self, options_window: &OptionsWindow) {
        self.init_force = Some(options_window.init_force);
        self.snapshot_force = Some(options_window.snapshot_force);
        self.snapshot_branch = Some(options_window.snapshot_branch);
        self.snapshot_ignore = match options_window.snapshot_ignore.is_empty() {
            true => None,
            false => Some(options_window.snapshot_ignore.clone()),
        };
        self.sync_type = Some(options_window.sync_type.clone());
        self.sync_no_checkout = Some(options_window.sync_no_checkout);
        self.sync_no_track = Some(options_window.sync_no_track);
        self.sync_thread = Some(options_window.sync_thread);
        self.fetch_thread = Some(options_window.fetch_thread);
        self.save_settings();
    }

    /// save ignore repositories from check box
    pub fn save_remove_common_ignore(&mut self, path: String, do_ignore: bool) {
        let mut common_ignore = self.common_ignore.clone().unwrap_or(String::new());
        if !do_ignore && common_ignore.contains(&path) {
            common_ignore = common_ignore.replace(&format!("{}\n", &path), "");
        } else if do_ignore && !common_ignore.contains(&path) {
            common_ignore.push_str(&format!("{}\n", &path));
        }
        self.common_ignore = Some(common_ignore);
        self.save_settings();
    }

    pub fn get_recent_projects(&self) -> Option<Vec<String>> {
        self.recent_projects.clone()
    }

    pub fn get_recent_configs(&self) -> Option<Vec<String>> {
        self.recent_configs.clone()
    }

    pub fn push_recent_project(&mut self, path: String) {
        let history = &mut self.recent_projects;
        TomlSetting::get_new_history(path, history);
        self.save_settings();
    }

    pub fn push_recent_config(&mut self, path: String) {
        let history = &mut self.recent_configs;
        TomlSetting::get_new_history(path, history);
        self.save_settings();
    }

    fn get_new_history(path: String, history: &mut Option<Vec<String>>) {
        if path.is_empty() {
            return;
        }

        if let Some(history) = history {
            // if contain the path, remove it
            if history.contains(&path) {
                let idx = history.iter().position(|r| r.to_owned() == path).unwrap();
                history.remove(idx);
            }
            // push front
            history.insert(0, path);

            // remove oldest history
            if history.len() > 20 {
                history.pop();
            }
        } else {
            *history = Some(vec![path]);
        }
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        // introduce
        out.push_str("# This file is about mgit-gui settings.\n");
        out.push_str("\n");

        // recent
        if self.recent_projects.is_some() || self.recent_configs.is_some() {
            out.push_str("# recent\n");
            // serialize recent projects
            if let Some(recent_projects) = &self.recent_projects {
                let mut recent_project_str = String::new();
                for recent_project in recent_projects {
                    recent_project_str =
                        format!("{}\n\t\"{}\",", recent_project_str, recent_project);
                }
                recent_project_str = format!("{}\n", recent_project_str);
                out.push_str(&format!("recent_projects = [{}]\n", recent_project_str));
                out.push_str("\n");
            }

            // serialize recent configs
            if let Some(recent_configs) = &self.recent_configs {
                let mut recent_config_str = String::new();
                for recent_config in recent_configs {
                    recent_config_str = format!("{}\n\t\"{}\",", recent_config_str, recent_config);
                }
                recent_config_str = format!("{}\n", recent_config_str);
                out.push_str(&format!("recent_configs = [{}]\n", recent_config_str));
            }
        }

        // options
        out.push_str("\n");
        out.push_str("# mgit options\n");
        let toml = toml_edit::ser::to_item(self).unwrap();
        if let Some(item) = toml.get("init_force") {
            out.push_str(&format!("init_force = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("snapshot_force") {
            out.push_str(&format!("snapshot_force = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("snapshot_branch") {
            out.push_str(&format!("snapshot_branch = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("snapshot_ignore") {
            out.push_str(&format!("snapshot_ignore = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("sync_type") {
            out.push_str(&format!("sync_type = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("sync_no_checkout") {
            out.push_str(&format!("sync_no_checkout = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("sync_no_track") {
            out.push_str(&format!("sync_no_track = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("sync_thread") {
            out.push_str(&format!("sync_thread = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("common_ignore") {
            out.push_str(&format!("common_ignore = {}\n", item));
            out.push_str("\n");
        }

        if let Some(item) = toml.get("fetch_thread") {
            out.push_str(&format!("fetch_thread = {}\n", item));
        }

        out
    }
}
