use std::path::PathBuf;

use filetime::FileTime;

use mgit::utils::path::PathExtension;

use crate::toml_settings::project_settings::{get_new_history, TomlProjectSettings};
use crate::toml_settings::user_settings::TomlUserSettings;

// ========================================
// recent project settings for app
// ========================================
impl super::Editor {
    pub(crate) fn load_setting(&mut self) {
        self.toml_user_settings = TomlUserSettings::load();
        self.load_recent_projects();

        // if app startup with args including project path, use the path
        if let Some(startup_project) = self.get_path_from_env_args() {
            self.project_path = startup_project.clone();
            self.push_recent_project();

            // load project settings
            self.load_project_settings();

            self.config_file = format!("{}/.gitrepos", startup_project);
            self.push_recent_config();
        }
        // if app startup normally, load saves
        else {
            // restore last project and settings
            if !self.recent_projects.is_empty() {
                self.project_path = self.recent_projects[0].to_owned();
            }

            // load project settings
            self.load_project_settings();

            // restore last config file
            if let Some(recent_configs) = &self.get_recent_configs() {
                if !recent_configs.is_empty() {
                    self.config_file = recent_configs[0].to_owned();
                }
            }
        }

        // restore options setting
        self.options_window
            .load_option_from_settings(&self.toml_user_settings, &self.get_snapshot_ignore());
    }

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
    pub(crate) fn save_ignore(&mut self, new_path: String, is_ignore: bool) {
        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_ignore(new_path.clone(), is_ignore);
            self.save_project_settings();
        }
    }

    pub(crate) fn get_ignores(&self) -> Option<Vec<String>> {
        self.toml_project_settings.ignore.as_ref().map(|content| {
            content
                .trim()
                .split('\n')
                .filter_map(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                })
                .collect()
        })
    }

    pub(crate) fn get_new_branch_ignores(&self) -> Option<Vec<String>> {
        self.toml_project_settings
            .new_branch_ignore
            .as_ref()
            .map(|content| {
                content
                    .trim()
                    .split('\n')
                    .filter_map(|s| {
                        let s = s.trim().to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    })
                    .collect()
            })
    }

    pub(crate) fn get_new_branch_name(&self) -> Option<String> {
        self.toml_project_settings.new_branch_name.clone()
    }

    pub(crate) fn get_new_branch_config_path(&self) -> Option<String> {
        self.toml_project_settings.new_branch_config_path.clone()
    }

    pub(crate) fn get_new_tag_ignores(&self) -> Option<Vec<String>> {
        self.toml_project_settings
            .new_tag_ignore
            .as_ref()
            .map(|content| {
                content
                    .trim()
                    .split('\n')
                    .filter_map(|s| {
                        let s = s.trim().to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    })
                    .collect()
            })
    }

    pub(crate) fn get_new_tag_name(&self) -> Option<String> {
        self.toml_project_settings.new_tag_name.clone()
    }

    pub(crate) fn get_new_tag_push(&self) -> bool {
        match self.toml_project_settings.new_tag_push {
            Some(flag) => flag,
            None => true,
        }
    }

    pub(crate) fn save_new_branch_option(&mut self) {
        let new_branch_ignore = self
            .new_branch_window
            .get_ignore_repos()
            .iter()
            .map(|s| s.display_path())
            .collect::<Vec<_>>()
            .join("\n");

        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_new_branch_ignore(new_branch_ignore.to_owned());

            let new_branch_name = self.new_branch_window.new_branch.clone();
            self.toml_project_settings
                .save_new_branch_name(new_branch_name);

            let new_config_path = self.new_branch_window.new_config_path.clone();
            self.toml_project_settings
                .save_new_branch_config_path(new_config_path);

            self.save_project_settings();
        }
    }

    pub(crate) fn save_new_tag_option(&mut self) {
        let new_tag_ignore = self
            .new_tag_window
            .get_ignore_repos()
            .iter()
            .map(|s| s.display_path())
            .collect::<Vec<_>>()
            .join("\n");

        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_new_tag_ignore(new_tag_ignore.to_owned());

            let new_tag = self.new_tag_window.new_tag.clone();
            self.toml_project_settings.save_new_tag_name(new_tag);

            let new_tag_push = self.new_tag_window.push.clone();
            self.toml_project_settings.save_new_tag_push(new_tag_push);

            self.save_project_settings();
        }
    }

    pub(crate) fn save_snapshot_ignore(&mut self) {
        let snapshot_ignore = &self.options_window.snapshot_ignore;
        if !self.recent_projects.is_empty() {
            self.toml_project_settings
                .save_snapshot_ignore(snapshot_ignore.to_owned());
            self.save_project_settings();
        }
    }

    pub(crate) fn get_snapshot_ignore(&self) -> Option<String> {
        self.toml_project_settings.snapshot_ignore.clone()
    }

    pub(crate) fn get_recent_projects(&self) -> Vec<String> {
        self.recent_projects.clone()
    }

    pub(crate) fn push_recent_project(&mut self) {
        let new_path = &self.project_path;
        if new_path.is_empty() {
            return;
        }

        let history = &mut self.recent_projects;
        get_new_history(new_path.to_owned(), history);
    }

    pub(crate) fn load_project_settings(&mut self) {
        let new_path = &self.project_path;
        if new_path.is_empty() {
            return;
        }

        self.toml_project_settings = TomlProjectSettings::load(new_path.to_owned(), false);
        self.toml_project_settings.project = Some(new_path.to_owned());
        self.toml_project_settings.save(new_path.to_owned());
    }

    pub(crate) fn get_recent_configs(&self) -> Option<Vec<String>> {
        self.toml_project_settings.recent_configs.clone()
    }

    pub(crate) fn push_recent_config(&mut self) {
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

    fn save_project_settings(&self) {
        self.toml_project_settings.save(self.project_path.clone());
    }

    // startup with arg: mgit-gui <path>
    fn get_path_from_env_args(&self) -> Option<String> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            let path = PathBuf::from(args[1].clone());
            if let Ok(path) = std::fs::canonicalize(path) {
                let path = (format!("{}", path.display())).norm_path();

                let norm_path = path.replace("//?/", "");
                return Some(norm_path);
            }
        }
        None
    }
}
