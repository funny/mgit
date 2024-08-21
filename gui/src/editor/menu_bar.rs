use std::path::Path;

use eframe::egui;

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

                // refresh button
                if ui.button("  New Branch").clicked() {
                    self.close_all_windows();
                    self.new_branch_window.update_repo(
                        &self.project_path,
                        &self.config_file,
                        &self.toml_config,
                    );
                    self.new_branch_is_open = true;
                    ui.close_menu();
                }

                ui.separator();
                // clean button - open ok/cancel dialog
                if ui.button("  Clean").clicked() {
                    self.close_all_windows();
                    self.clean_is_open = true;
                    ui.close_menu();
                }
            });

            // Settings menu
            ui.menu_button("Settings", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // option button
                if ui.button("  Options").clicked() {
                    self.close_all_windows();
                    self.options_is_open = true;
                    ui.close_menu();
                }

                // theme button
                ui.menu_button("  Theme", |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // about button
                if ui.button("  About").clicked() {
                    self.close_all_windows();
                    self.about_is_open = true;
                    ui.close_menu();
                }
            });
        });
    }
}
