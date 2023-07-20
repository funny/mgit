use serde::{Deserialize, Serialize};

use crate::editor::window::options::OptionsWindow;
use crate::toml_settings::SyncType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TomlUserSettings {
    version: Option<String>,

    // options
    pub init_force: Option<bool>,
    pub snapshot_force: Option<bool>,
    pub snapshot_branch: Option<bool>,
    pub sync_type: Option<SyncType>,
    pub sync_no_checkout: Option<bool>,
    pub sync_no_track: Option<bool>,
    pub sync_thread: Option<u32>,
    pub sync_depth: Option<u32>,
    pub fetch_thread: Option<u32>,
    pub fetch_depth: Option<u32>,
}

impl Default for TomlUserSettings {
    fn default() -> Self {
        Self {
            version: Some(String::from(std::env!("CARGO_PKG_VERSION"))),

            // options
            init_force: Some(true),
            snapshot_force: Some(true),
            snapshot_branch: Some(true),
            sync_type: Some(SyncType::Stash),
            sync_no_checkout: Some(false),
            sync_no_track: Some(false),
            sync_thread: Some(4),
            sync_depth: None,
            fetch_thread: Some(4),
            fetch_depth: None,
        }
    }
}

impl TomlUserSettings {
    /// load settings from toml file
    /// create new one if toml file doesn't exsit
    pub fn load() -> TomlUserSettings {
        if let Some(path) = home::home_dir() {
            let setting_file = path.join(".mgit/settings.toml");
            if setting_file.is_file() {
                let txt = std::fs::read_to_string(setting_file).unwrap();
                let user_settings: TomlUserSettings = toml::from_str(txt.as_str()).unwrap();
                return user_settings;
            }
        }
        let mut user_settings = TomlUserSettings::default();
        user_settings.save();
        user_settings
    }

    /// save settings into toml file
    pub fn save(&mut self) {
        if let Some(path) = home::home_dir() {
            let user_settings_file = path.join(".mgit/settings.toml");
            let toml_string = self.serialize();

            let _ = std::fs::create_dir_all(path.join(".mgit"));
            std::fs::write(user_settings_file, toml_string)
                .expect("Failed to write file settings.toml!");
        }
    }

    /// save_option from options window
    pub fn save_options(&mut self, options_window: &OptionsWindow) {
        self.init_force = Some(options_window.init_force);

        self.snapshot_force = Some(options_window.snapshot_force);
        self.snapshot_branch = Some(options_window.snapshot_branch);

        self.sync_type = Some(options_window.sync_type);
        self.sync_no_checkout = Some(options_window.sync_no_checkout);
        self.sync_no_track = Some(options_window.sync_no_track);
        self.sync_thread = Some(options_window.sync_thread);
        self.sync_depth = match options_window.sync_depth > 0 {
            true => Some(options_window.sync_depth),
            false => None,
        };

        self.fetch_thread = Some(options_window.fetch_thread);
        self.fetch_depth = match options_window.fetch_depth > 0 {
            true => Some(options_window.fetch_depth),
            false => None,
        };

        self.save();
    }

    fn serialize(&mut self) -> String {
        let mut out = String::new();
        // introduce
        out.push_str("# This file is about mgit-gui settings.\n");

        if let Ok(toml_string) = toml_edit::ser::to_string_pretty(&self) {
            out.push('\n');
            out.push_str(&toml_string)
        }
        out
    }
}
