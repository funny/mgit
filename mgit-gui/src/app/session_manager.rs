use crate::configs::project::{get_new_history, TomlProjectSettings};
use crate::configs::user::TomlUserSettings;
use crate::ui::windows::NewBranchWindow;
use crate::ui::windows::NewTagWindow;
use filetime::FileTime;
use mgit::utils::path::PathExtension;
use std::path::PathBuf;

pub struct SessionManager {
    pub project_path: String,
    pub config_file: String,

    pub user_settings: TomlUserSettings,
    pub project_settings: TomlProjectSettings,
    pub recent_projects: Vec<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            project_path: String::new(),
            config_file: String::new(),
            user_settings: TomlUserSettings::default(),
            project_settings: TomlProjectSettings::default(),
            recent_projects: Vec::new(),
        }
    }

    pub fn load_setting(&mut self) {
        self.user_settings = TomlUserSettings::load();
        self.load_recent_projects();

        // if app startup with args including project path, use the path
        if let Some(startup_project) = get_path_from_env_args() {
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
    }

    pub fn save_ignore(&mut self, new_path: String, is_ignore: bool) {
        if !self.recent_projects.is_empty() {
            self.project_settings
                .save_ignore(new_path.clone(), is_ignore);
            self.save_project_settings();
        }
    }

    pub fn get_ignores(&self) -> Option<Vec<String>> {
        self.project_settings.ignore.as_ref().map(|content| {
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
            // Some(vec![])
        })
    }

    pub fn get_labels(&self) -> Option<Vec<String>> {
        if self.project_settings.labels.is_empty() {
            None
        } else {
            Some(self.project_settings.labels.iter().cloned().collect())
        }
    }

    pub fn get_new_branch_ignores(&self) -> Option<Vec<String>> {
        self.project_settings
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
                // Some(vec![])
            })
    }

    pub fn get_new_branch_name(&self) -> Option<String> {
        self.project_settings.new_branch_name.clone()
    }

    pub fn get_new_branch_config_path(&self) -> Option<String> {
        self.project_settings.new_branch_config_path.clone()
    }

    pub fn get_new_tag_ignores(&self) -> Option<Vec<String>> {
        self.project_settings
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
                // Some(vec![])
            })
    }

    pub fn get_new_tag_name(&self) -> Option<String> {
        self.project_settings.new_tag_name.clone()
    }

    pub fn get_new_tag_push(&self) -> bool {
        self.project_settings.new_tag_push.unwrap_or(true)
    }

    pub fn save_new_branch_option_from_window(&mut self, window: &NewBranchWindow) {
        let new_branch_ignore = window
            .get_ignore_repos()
            .iter()
            .map(|s| s.display_path())
            .collect::<Vec<_>>()
            .join("\n");

        if !self.recent_projects.is_empty() {
            self.project_settings
                .save_new_branch_ignore(new_branch_ignore.to_owned());

            let new_branch_name = window.new_branch.clone();
            self.project_settings.save_new_branch_name(new_branch_name);

            let new_config_path = window.new_config_path.clone();
            self.project_settings
                .save_new_branch_config_path(new_config_path);

            self.save_project_settings();
        }
    }

    pub fn save_new_tag_option_from_window(&mut self, window: &NewTagWindow) {
        let new_tag_ignore = window
            .get_ignore_repos()
            .iter()
            .map(|s| s.display_path())
            .collect::<Vec<_>>()
            .join("\n");

        if !self.recent_projects.is_empty() {
            self.project_settings
                .save_new_tag_ignore(new_tag_ignore.to_owned());

            let new_tag = window.new_tag.clone();
            self.project_settings.save_new_tag_name(new_tag);

            let new_tag_push = window.push;
            self.project_settings.save_new_tag_push(new_tag_push);

            self.save_project_settings();
        }
    }

    pub fn save_snapshot_ignore(&mut self, snapshot_ignore: String) {
        if !self.recent_projects.is_empty() {
            self.project_settings
                .save_snapshot_ignore(snapshot_ignore.to_owned());
            self.save_project_settings();
        }
    }

    pub fn get_snapshot_ignore(&self) -> Option<String> {
        self.project_settings.snapshot_ignore.clone()
    }

    pub fn load_project_settings(&mut self) {
        let new_path = &self.project_path;
        if new_path.is_empty() {
            return;
        }

        self.project_settings = TomlProjectSettings::load(new_path.to_owned(), false);
        self.project_settings.project = Some(new_path.to_owned());
        self.project_settings.save(new_path.to_owned());
    }

    pub fn get_recent_configs(&self) -> Option<Vec<String>> {
        self.project_settings.recent_configs.clone()
    }

    pub fn push_recent_config(&mut self) {
        let new_path = &self.config_file;
        if new_path.is_empty() {
            return;
        }

        if !self.recent_projects.is_empty() {
            self.project_settings
                .push_recent_config(new_path.to_owned());
            self.project_settings.save(self.recent_projects[0].clone());
        }
    }

    pub fn save_project_settings(&self) {
        self.project_settings.save(self.project_path.clone());
    }

    // From history.rs
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
            let project_settings = TomlProjectSettings::load(file_name.clone(), true);

            if let Some(project) = &project_settings.project {
                self.recent_projects.push(project.to_owned());
            }
        }
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
}

// startup with arg: mgit-gui <path>
fn get_path_from_env_args() -> Option<String> {
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
