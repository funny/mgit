use eframe::egui;
use eframe::egui::{CornerRadius, Frame, Margin, Stroke};
use tracing::info;

use mgit::utils::path::PathExtension;

use crate::app::events::{Event, InputEvent};
use crate::app::GuiApp;
use crate::ui::components::ProgressBar;
use crate::ui::panels::{
    ConfigurationPanel, LabelsPanel, MenuBarPanel, QuickBarPanel, RepositoriesPanel,
};
use crate::ui::style::DEFAULT_WIDTH;
use crate::utils::system::{open_in_file_explorer, open_in_file_explorer_select};

// ========================================
// ui design for app
// ========================================
impl GuiApp {
    /// quick bar panel of app
    pub(crate) fn top_view(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mgit_gui_top_bar")
            .frame(Frame {
                inner_margin: Margin::symmetric(8, 2),
                corner_radius: CornerRadius::ZERO,
                fill: ctx.style().visuals.window_fill(),
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.add_enabled_ui(!self.windows.is_error_open(), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 5.0);

                    MenuBarPanel::show(ui, self);

                    let event_tx = self.app_context.event_tx.clone();
                    let mut sync_hard_dialog_open = self.windows.sync_hard_dialog_open;

                    QuickBarPanel::show(ui, &event_tx, &mut sync_hard_dialog_open, || {
                        self.windows.close_all();
                    });
                    self.windows.sync_hard_dialog_open = sync_hard_dialog_open;

                    ui.add_space(2.0);

                    ui.add(ProgressBar::new(
                        self.app_context.repo_manager.progress.clone(),
                        &self.app_context.repo_manager.repo_states,
                        self.context.clone(),
                    ));
                });
            });
    }

    /// content_view of app
    pub(crate) fn content_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.windows.is_error_open(), |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.set_min_width(DEFAULT_WIDTH);
                    ui.strong("Configuration");

                    let recent_projects = self.app_context.session_manager.get_recent_projects();
                    let recent_configs = self.app_context.session_manager.get_recent_configs();

                    let mut is_project_changed = false;
                    let mut is_config_changed = false;

                    let out = ConfigurationPanel::show(
                        ui,
                        &mut self.app_context.session_manager.project_path,
                        &mut self.app_context.session_manager.config_file,
                        &recent_projects,
                        recent_configs.as_deref(),
                    );

                    {
                        if let Some(path) = ui
                            .ctx()
                            .input(|i| i.raw.dropped_files.first().and_then(|x| x.path.clone()))
                        {
                            self.app_context.session_manager.config_file =
                                path.as_path().norm_path();
                            is_config_changed = true;
                        }
                    }

                    if out.pick_project_dir {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.app_context.session_manager.project_path =
                                path.as_path().norm_path();
                            info!(
                                project_path =
                                    self.app_context.session_manager.project_path.as_str(),
                                "ui_pick_project_dir"
                            );
                            is_project_changed = true;
                        } else {
                            info!("ui_pick_project_dir_canceled");
                        }
                    }

                    if out.open_project_dir {
                        info!(
                            project_path = self.app_context.session_manager.project_path.as_str(),
                            "ui_open_project_dir"
                        );
                        open_in_file_explorer(&self.app_context.session_manager.project_path);
                    }

                    if out.pick_config_file {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.app_context.session_manager.config_file =
                                path.as_path().norm_path();
                            info!(
                                config_file = self.app_context.session_manager.config_file.as_str(),
                                "ui_pick_config_file"
                            );
                            is_config_changed = true;
                        } else {
                            info!("ui_pick_config_file_canceled");
                        }
                    }

                    if out.open_config_dir {
                        info!(
                            config_file = self.app_context.session_manager.config_file.as_str(),
                            "ui_open_config_dir"
                        );
                        open_in_file_explorer_select(&self.app_context.session_manager.config_file);
                    }

                    is_project_changed |= out.project_changed;
                    is_config_changed |= out.config_changed;

                    if is_project_changed {
                        self.enqueue_event(Event::Input(InputEvent::ProjectPathChanged(
                            self.app_context.session_manager.project_path.clone(),
                        )));
                    } else if is_config_changed {
                        self.enqueue_event(Event::Input(InputEvent::ConfigFileChanged(
                            self.app_context.session_manager.config_file.clone(),
                        )));
                    }

                    ui.separator();

                    let repos_count = match &self.app_context.repo_manager.mgit_config.repos {
                        Some(repo_configs) => repo_configs.len(),
                        _ => 0,
                    };

                    LabelsPanel::show(ui, self);

                    ui.strong(format!("Repositories ({})", repos_count));
                    RepositoriesPanel::show(ui, self);
                });
            });
        });
    }
}
