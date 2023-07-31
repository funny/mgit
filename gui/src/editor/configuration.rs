use std::path::{Path, PathBuf};

use eframe::egui;

use mgit::core::repos::{load_config, TomlConfig};
use mgit::utils::path::PathExtension;

use crate::editor::misc::open_in_file_explorer;
use crate::editor::ops::RepoState;
use crate::editor::window::options::OptionsWindow;
use crate::editor::Editor;
use crate::utils::command::CommandType;
use crate::utils::defines::hex_code;

impl Editor {
    pub(crate) fn load_config(&mut self) {
        let config_file = PathBuf::from(&self.config_file);
        if config_file.is_file() {
            if let Some(toml_config) = load_config(&config_file) {
                self.toml_config = toml_config;
                // init repo states and sync ignore
                if let Some(toml_repos) = &self.toml_config.repos {
                    let ignores = self.get_ignores().unwrap_or(vec![]);
                    toml_repos.iter().for_each(|toml_repo| {
                        let rel_path = toml_repo
                            .local
                            .as_ref()
                            .map_or(String::from("invalid"), |p| p.clone());

                        // get ignore state
                        let do_ignore = ignores.contains(&rel_path.display_path());

                        // init repo state
                        self.repo_states.push(RepoState {
                            no_ignore: !do_ignore,
                            ..RepoState::default()
                        });
                    });
                    self.ops_message_collector.update(toml_repos);
                }
            }
        }
    }

    pub(crate) fn clear_toml_config(&mut self) {
        self.toml_config = TomlConfig::default();
    }

    /// part of app/content_view
    pub(crate) fn configuration_panel(&mut self, ui: &mut egui::Ui) {
        let desired_width = ui.ctx().used_size().x - 192.0;

        egui::Grid::new("config_grid")
            .num_columns(3)
            .spacing([10.0, 4.0])
            .min_col_width(50.0)
            .max_col_width(desired_width)
            .min_row_height(20.0)
            .striped(false)
            .show(ui, |ui| {
                // project path
                ui.label("project");

                let mut is_project_changed = false;
                let mut is_config_changed = false;

                // combo box to select recent project
                egui::ComboBox::from_id_source("project_path")
                    .width(desired_width)
                    .show_ui(ui, |ui| {
                        for recent_project in &self.get_recent_projects() {
                            if ui.selectable_label(false, recent_project).clicked() {
                                self.project_path = recent_project.to_owned();
                                is_project_changed = true;
                            }
                        }
                    });

                // button to pick folder
                if ui.button(format!("{} open", hex_code::FOLDER)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.project_path = path.display().to_string().norm_path();
                        is_project_changed = true;
                    }
                }

                // button to open in file in explorer
                if ui
                    .add_sized([18.0, 18.0], egui::Button::new(hex_code::LINK_EXTERNAL))
                    .clicked()
                    && Path::new(&self.project_path).is_dir()
                {
                    open_in_file_explorer(self.project_path.clone());
                }

                // edit text for project
                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(66.0, 0.0),
                    egui::vec2(desired_width - 15.0, 20.0),
                );
                let project_edit_text = ui.put(
                    widget_rect,
                    egui::TextEdit::singleline(&mut self.project_path),
                );
                // key down - enter
                if project_edit_text.lost_focus() {
                    if ui.input().key_pressed(egui::Key::Enter) {
                        is_project_changed = true;

                        // close combo box
                        ui.memory().close_popup();
                    } else if ui.input().key_pressed(egui::Key::Tab) {
                        ui.memory().close_popup();
                    }
                };

                // if project_path changed , auto change config_file,
                if is_project_changed {
                    self.project_path = self.project_path.norm_path();

                    is_config_changed = true;
                    // save recent project
                    self.push_recent_project();

                    // reload project settings
                    self.load_project_settings();

                    // reload options setting
                    self.options_window = OptionsWindow::default();
                    self.options_window.load_option_from_settings(
                        &self.toml_user_settings,
                        &self.get_snapshot_ignore(),
                    );

                    self.config_file = format!("{}/.gitrepos", &self.project_path);
                }
                ui.end_row();

                // config file
                ui.label("config");

                // combo box to select rencet config file
                egui::ComboBox::from_id_source("config_file")
                    .width(desired_width)
                    .show_ui(ui, |ui| {
                        if let Some(recent_configs) = &self.get_recent_configs() {
                            for recent_config in recent_configs {
                                if ui.selectable_label(false, recent_config).clicked() {
                                    self.config_file = recent_config.to_owned();
                                    is_config_changed = true;
                                }
                            }
                        }
                    });

                // button to pick config file
                if ui.button(format!("{} open", hex_code::FILE)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.config_file = path.display().to_string().norm_path();
                        is_config_changed = true;
                    }
                }

                // button to open in file in explorer
                if ui
                    .add_sized([18.0, 18.0], egui::Button::new(hex_code::LINK_EXTERNAL))
                    .clicked()
                {
                    if let Some(path) = Path::new(&self.config_file).parent() {
                        if path.is_dir() {
                            open_in_file_explorer(path.to_str().unwrap().to_string());
                        }
                    }
                }

                // edit text for config file path
                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(66.0, 24.0),
                    egui::vec2(desired_width - 15.0, 20.0),
                );
                let config_edit_text = ui.put(
                    widget_rect,
                    egui::TextEdit::singleline(&mut self.config_file),
                );
                // key down - enter
                if config_edit_text.lost_focus() {
                    self.config_file = self.config_file.norm_path();
                    if ui.input().key_pressed(egui::Key::Enter) {
                        is_config_changed = true;

                        // close combo box
                        ui.memory().close_popup();
                    } else if ui.input().key_pressed(egui::Key::Tab) {
                        ui.memory().close_popup();
                    }
                };

                // if config_file changed, auto refresh
                if is_config_changed {
                    if Path::new(&self.config_file).is_file() {
                        self.push_recent_config();
                    }

                    self.exec_ops(CommandType::Refresh);
                }
                ui.end_row();
            });
    }
}
