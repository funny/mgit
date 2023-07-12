use super::options_window::OptionsWindow;
use filetime::FileTime;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::path::PathBuf;
use toml;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SyncType {
    Normal,
    Stash,
    Hard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TomlProjectSettings {
    pub project: Option<String>,
    pub recent_configs: Option<Vec<String>>,
    pub snapshot_ignore: Option<String>,
    // --ignore for fetch, sync and track
    pub ignore: Option<String>,
}

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

impl Default for TomlProjectSettings {
    fn default() -> Self {
        Self {
            project: None,
            recent_configs: None,
            snapshot_ignore: None,
            ignore: None,
        }
    }
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
            std::fs::write(&recent_file, toml_string).expect("Failed to write file");
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
        if !is_ignore && ignore.contains(&path) {
            ignore = ignore.replace(&format!("{}\n", &path), "");
        } else if is_ignore && !ignore.contains(&path) {
            ignore.push_str(&format!("{}\n", &path));
        }

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

    fn serialize(&self) -> String {
        let mut out = String::new();
        // introduce
        out.push_str("# This file is about recent config.\n");

        if let Ok(toml_string) = toml_edit::ser::to_string_pretty(&self) {
            out.push_str("\n");
            out.push_str(&toml_string)
        }
        out
    }
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

        self.sync_type = Some(options_window.sync_type.clone());
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
            out.push_str("\n");
            out.push_str(&toml_string)
        }
        out
    }
}

// ========================================
// recent project settings for app
// ========================================
impl super::App {
    pub fn load_recent_projects(&mut self) {
        let tmp_dir: PathBuf;
        if let Some(path) = home::home_dir() {
            tmp_dir = path.join(".mgit/tmp");
        } else {
            return;
        }

        // if dir is null, return
        if !tmp_dir.is_dir() {
            return;
        }

        // get all files in ".mgit/tmp"
        let mut entries: Vec<_> = tmp_dir.read_dir().unwrap().map(|r| r.unwrap()).collect();
        // sort by file's last modifcation time
        entries.sort_by(|a, b| {
            let a_meta_data = std::fs::metadata(a.path()).unwrap();
            let a_last_modification_time = FileTime::from_last_modification_time(&a_meta_data);

            let b_meta_data = std::fs::metadata(b.path()).unwrap();
            let b_last_modification_time = FileTime::from_last_modification_time(&b_meta_data);
            b_last_modification_time
                .partial_cmp(&a_last_modification_time)
                .unwrap()
        });

        self.recent_projects = Vec::new();
        // construct recent projects
        for entry in entries {
            let file_name = entry.file_name().into_string().unwrap();
            let toml_project_settings = TomlProjectSettings::load(file_name.clone(), true);

            if let Some(project) = &toml_project_settings.project {
                self.recent_projects.push(project.to_owned());
            }
        }
    }

    /// save ignore repositories from check box
    pub fn save_ignore(&mut self, new_path: String, is_ignore: bool) {
        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_ignore(new_path.clone(), is_ignore);
            self.save_project_settings();
        }
    }

    pub fn get_ignore(&self) -> Option<String> {
        self.toml_project_settings.ignore.clone()
    }

    pub fn get_ignores(&self) -> Option<Vec<String>> {
        self.toml_project_settings
            .ignore
            .as_ref()
            .map(|content| content.split('\n').map(|s| s.to_string()).collect())
    }

    pub fn save_snapshot_ignore(&mut self) {
        let snapshot_ignore = &self.options_window.snapshot_ignore;
        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_snapshot_ignore(snapshot_ignore.to_owned());
            self.save_project_settings();
        }
    }

    pub fn get_snapshot_ignore(&self) -> Option<String> {
        self.toml_project_settings.snapshot_ignore.clone()
    }

    pub fn get_recent_projects(&self) -> Vec<String> {
        self.recent_projects.clone()
    }

    pub fn push_recent_project(&mut self) {
        let new_path = &self.project_path;
        if new_path.is_empty() {
            return;
        }

        let history = &mut self.recent_projects;
        get_new_history(new_path.to_owned(), history);
    }

    pub fn load_project_settings(&mut self) {
        let new_path = &self.project_path;
        if new_path.is_empty() {
            return;
        }

        self.toml_project_settings = TomlProjectSettings::load(new_path.to_owned(), false);
        self.toml_project_settings.project = Some(new_path.to_owned());
        self.toml_project_settings.save(new_path.to_owned());
    }
    pub fn get_recent_configs(&self) -> Option<Vec<String>> {
        self.toml_project_settings.recent_configs.clone()
    }

    pub fn push_recent_config(&mut self) {
        let new_path = &self.config_file;
        if new_path.is_empty() {
            return;
        }

        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .push_recent_config(new_path.to_owned());
            self.toml_project_settings
                .save(self.recent_projects[0].clone());
        }
    }

    pub fn save_project_settings(&self) {
        self.toml_project_settings.save(self.project_path.clone());
    }
}

fn get_new_history(path: String, history: &mut Vec<String>) {
    if !history.is_empty() {
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
        *history = vec![path];
    }
}
