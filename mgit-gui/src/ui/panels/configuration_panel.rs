use std::path::Path;

use eframe::egui;

use crate::ui::style::hex_code;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ConfigurationPanelOutput {
    pub(crate) project_changed: bool,
    pub(crate) config_changed: bool,
    pub(crate) pick_project_dir: bool,
    pub(crate) pick_config_file: bool,
    pub(crate) open_project_dir: bool,
    pub(crate) open_config_dir: bool,
}

pub(crate) struct ConfigurationPanel;

impl ConfigurationPanel {
    pub(crate) fn show(
        ui: &mut egui::Ui,
        project_path: &mut String,
        config_file: &mut String,
        recent_projects: &[String],
        recent_configs: Option<&[String]>,
    ) -> ConfigurationPanelOutput {
        let mut out = ConfigurationPanelOutput::default();

        ui.horizontal(|ui| {
            ui.add_sized([72.0, 20.0], egui::Label::new("project"));
            ui.style_mut().spacing.item_spacing.x = 1.0;
            let project_edit = ui.add_sized(
                [ui.clip_rect().max.x - ui.cursor().min.x - 80.0, 20.0],
                egui::TextEdit::singleline(project_path),
            );
            if project_edit.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    out.project_changed = true;
                    ui.close();
                } else if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                    ui.close();
                }
            }
            ui.style_mut().spacing.item_spacing.x = 5.0;

            let popup_id = ui.make_persistent_id("project_history_popup");
            let btn_response = ui.button(hex_code::DROPDOWN);
            if btn_response.clicked() {
                #[allow(deprecated)]
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }

            #[allow(deprecated)]
            egui::popup_below_widget(
                ui,
                popup_id,
                &project_edit,
                egui::PopupCloseBehavior::CloseOnClick,
                |ui| {
                    ui.set_min_width(project_edit.rect.width());
                    for recent_project in recent_projects {
                        if ui
                            .selectable_value(project_path, recent_project.clone(), recent_project)
                            .clicked()
                        {
                            out.project_changed = true;
                            ui.close();
                        }
                    }
                },
            );

            if ui.button(format!("{}", hex_code::FOLDER)).clicked() {
                out.pick_project_dir = true;
            }

            if ui
                .add_sized([18.0, 18.0], egui::Button::new(hex_code::LINK_EXTERNAL))
                .clicked()
                && Path::new(project_path).is_dir()
            {
                out.open_project_dir = true;
            }

            ui.add_space(10.0);
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_sized([72.0, 20.0], egui::Label::new("config"));
            ui.style_mut().spacing.item_spacing.x = 1.0;

            let config_edit = ui.add_sized(
                [ui.clip_rect().max.x - ui.cursor().min.x - 80.0, 20.0],
                egui::TextEdit::singleline(config_file),
            );
            if config_edit.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    out.config_changed = true;
                    ui.close();
                } else if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                    ui.close();
                }
            }
            ui.style_mut().spacing.item_spacing.x = 5.0;

            let popup_id = ui.make_persistent_id("config_history_popup");
            let btn_response = ui.button(hex_code::DROPDOWN);
            if btn_response.clicked() {
                #[allow(deprecated)]
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }

            #[allow(deprecated)]
            egui::popup_below_widget(
                ui,
                popup_id,
                &config_edit,
                egui::PopupCloseBehavior::CloseOnClick,
                |ui| {
                    ui.set_min_width(config_edit.rect.width());
                    if let Some(recent_configs) = recent_configs {
                        for recent_config in recent_configs {
                            if ui
                                .selectable_value(config_file, recent_config.clone(), recent_config)
                                .clicked()
                            {
                                out.config_changed = true;
                                ui.close();
                            }
                        }
                    }
                },
            );

            if ui.button(format!("{}", hex_code::FILE)).clicked() {
                out.pick_config_file = true;
            }

            if ui
                .add_sized([18.0, 18.0], egui::Button::new(hex_code::LINK_EXTERNAL))
                .clicked()
            {
                out.open_config_dir = true;
            }

            ui.add_space(10.0);
        });

        out
    }
}
