use std::path::Path;

use eframe::{egui, Theme};

use egui::Ui;
use mgit::utils::path::PathExtension;

use crate::editor::Editor;
use crate::utils::command::CommandType;
use crate::utils::defines::MENU_BOX_WIDTH;

impl Editor {
    pub(crate) fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            // Commands menu
            ui.menu_button("Commands", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // init button
                if ui.button("  Init").clicked() {
                    self.exec_ops(CommandType::Init);
                    ui.close_menu();
                }

                // snapshot button - open save file dialog
                if ui.button("  Snapshot").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(Path::new(&self.project_path.replace('/', "\\")))
                        .set_title("save new config file")
                        .set_file_name(".gitrepos")
                        .save_file()
                    {
                        self.config_file = path.to_str().unwrap().to_string().norm_path();
                        self.exec_ops(CommandType::Snapshot);
                    }
                    ui.close_menu();
                }

                ui.separator();

                // refresh button
                if ui.button("  Refresh").clicked() {
                    self.exec_ops(CommandType::Refresh);
                    ui.close_menu();
                }

                // fetch button
                if ui.button("  Fetch").clicked() {
                    self.exec_ops(CommandType::Fetch);
                    ui.close_menu();
                }

                // sync button
                if ui.button("  Sync").clicked() {
                    self.exec_ops(CommandType::Sync);
                    ui.close_menu();
                }

                // track button
                if ui.button("  Track").clicked() {
                    self.exec_ops(CommandType::Track);
                    ui.close_menu();
                }

                ui.separator();

                // new branch button
                if ui.button("  New Branch").clicked() {
                    self.close_all_windows();

                    let new_branch_ignore = self.get_new_branch_ignores().unwrap_or(Vec::new());
                    let new_branch = self.get_new_branch_name().unwrap_or(String::new());
                    let new_config_path =
                        self.get_new_branch_config_path().unwrap_or(String::new());

                    self.new_branch_window.update_settings(
                        &self.project_path,
                        &self.config_file,
                        &self.toml_config,
                        new_branch,
                        new_config_path,
                        &new_branch_ignore,
                    );

                    self.show_new_branch_window = true;
                    ui.close_menu();
                }

                // new tag button
                if ui.button("  New Tag").clicked() {
                    self.close_all_windows();

                    let new_tag_ignore = self.get_new_tag_ignores().unwrap_or(Vec::new());
                    let new_tag = self.get_new_tag_name().unwrap_or(String::new());
                    let new_tag_push = self.get_new_tag_push();

                    self.new_tag_window.update_settings(
                        &self.project_path,
                        &self.config_file,
                        &self.toml_config,
                        new_tag,
                        new_tag_push,
                        &new_tag_ignore,
                    );

                    self.show_new_tag_window = true;
                    ui.close_menu();
                }

                ui.separator();
                // clean button - open ok/cancel dialog
                if ui.button("  Clean").clicked() {
                    self.close_all_windows();
                    self.show_clean_dialog = true;
                    ui.close_menu();
                }
            });

            // Settings menu
            ui.menu_button("Settings", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // option button
                if ui.button("  Options").clicked() {
                    self.close_all_windows();
                    self.show_options_window = true;
                    ui.close_menu();
                }

                // theme button
                ui.menu_button("  Theme", |ui| {
                    if let Some(theme) = global_dark_light_mode_buttons(ui) {
                        self.toml_user_settings.theme = Some(theme);
                        self.toml_user_settings.save_options(&self.options_window);
                    }
                });
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // about button
                if ui.button("  About").clicked() {
                    self.close_all_windows();
                    self.show_about_window = true;
                    ui.close_menu();
                }
            });
        });
    }
}

fn global_dark_light_mode_buttons(ui: &mut Ui) -> Option<Theme> {
    let mut visuals = (*ui.ctx().style()).visuals.clone();
    let mut clicked = false;
    ui.horizontal(|ui| {
        clicked |= ui
            .selectable_value(&mut visuals, egui::Visuals::light(), "☀ Light")
            .clicked();
        clicked |= ui
            .selectable_value(&mut visuals, egui::Visuals::dark(), "🌙 Dark")
            .clicked();
    });

    if !clicked {
        return None;
    }

    let dark_mode = visuals.dark_mode;
    ui.ctx().set_visuals(visuals);

    if dark_mode {
        Some(Theme::Dark)
    } else {
        Some(Theme::Light)
    }
}
