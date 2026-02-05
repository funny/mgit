use eframe::egui;

use crate::app::GuiApp;

pub(crate) struct LabelsPanel;

impl LabelsPanel {
    pub(crate) fn show(ui: &mut egui::Ui, app: &mut GuiApp) {
        if let Some(repos) = &app.app_context.repo_manager.mgit_config.repos {
            let mut labels = mgit::utils::label::collect(repos);
            if labels.is_empty() {
                return;
            }

            ui.strong("Labels");
            labels.insert("none");
            app.app_context
                .session_manager
                .project_settings
                .labels
                .retain(|x| labels.contains(x.as_str()));

            let mut changed = false;
            ui.horizontal(|ui| {
                for label in labels {
                    let contains = app
                        .app_context
                        .session_manager
                        .project_settings
                        .labels
                        .contains(label);
                    let mut checked = contains;
                    ui.checkbox(&mut checked, label);
                    if checked == contains {
                        continue;
                    }

                    changed = true;
                    if checked {
                        app.app_context
                            .session_manager
                            .project_settings
                            .labels
                            .insert(label.to_string());
                    } else {
                        app.app_context
                            .session_manager
                            .project_settings
                            .labels
                            .remove(label);
                    }
                }
            });

            if changed {
                app.app_context.session_manager.save_project_settings();
                app.app_context.repo_manager.recompute_repo_filters(
                    app.app_context.session_manager.get_ignores().as_ref(),
                    app.app_context.session_manager.get_labels().as_ref(),
                );
            }
            ui.separator();
        }
    }
}
