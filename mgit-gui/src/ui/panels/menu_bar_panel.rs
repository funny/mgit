use std::path::Path;

use eframe::egui;
use tracing::info;

use mgit::utils::path::PathExtension;

use crate::app::events::{Action, CommandType, Event, OpsCommand};
use crate::app::GuiApp;
use crate::ui::style::MENU_BOX_WIDTH;

pub(crate) struct MenuBarPanel;

impl MenuBarPanel {
    pub(crate) fn show(ui: &mut egui::Ui, app: &mut GuiApp) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Commands", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                if ui.button("  Init").clicked() {
                    info!("ui_click_menu_init");
                    app.enqueue_event(Event::Action(Action::RunOps(CommandType::Init.into())));
                    ui.close();
                }

                if ui.button("  Snapshot").clicked() {
                    info!("ui_click_menu_snapshot_open_dialog");
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(Path::new(
                            &app.app_context
                                .session_manager
                                .project_path
                                .replace('/', "\\"),
                        ))
                        .set_title("save new config file")
                        .set_file_name(".gitrepos")
                        .save_file()
                    {
                        let config_file = path.to_str().unwrap().to_string().norm_path();
                        info!(
                            config_file = config_file.as_str(),
                            "ui_menu_snapshot_selected_path"
                        );
                        app.enqueue_event(Event::Action(Action::RunOps(OpsCommand::Snapshot {
                            config_file,
                        })));
                    } else {
                        info!("ui_menu_snapshot_canceled");
                    }
                    ui.close();
                }

                ui.separator();

                if ui.button("  Refresh").clicked() {
                    info!("ui_click_menu_refresh");
                    app.enqueue_event(Event::Action(Action::Refresh));
                    ui.close();
                }

                if ui.button("  Fetch").clicked() {
                    info!("ui_click_menu_fetch");
                    app.enqueue_event(Event::Action(Action::RunOps(CommandType::Fetch.into())));
                    ui.close();
                }

                if ui.button("  Sync").clicked() {
                    info!("ui_click_menu_sync");
                    app.enqueue_event(Event::Action(Action::RunOps(CommandType::Sync.into())));
                    ui.close();
                }

                if ui.button("  Track").clicked() {
                    info!("ui_click_menu_track");
                    app.enqueue_event(Event::Action(Action::RunOps(CommandType::Track.into())));
                    ui.close();
                }

                ui.separator();

                if ui.button("  New Branch").clicked() {
                    info!("ui_click_menu_new_branch_open");
                    app.close_all_windows();

                    let new_branch_ignore = app
                        .app_context
                        .session_manager
                        .get_new_branch_ignores()
                        .unwrap_or_default();
                    let new_branch = app
                        .app_context
                        .session_manager
                        .get_new_branch_name()
                        .unwrap_or_default();
                    let new_config_path = app
                        .app_context
                        .session_manager
                        .get_new_branch_config_path()
                        .unwrap_or_default();

                    app.windows.new_branch.update_settings(
                        &app.app_context.session_manager.project_path,
                        &app.app_context.session_manager.config_file,
                        &app.app_context.repo_manager.mgit_config,
                        new_branch,
                        new_config_path,
                        &new_branch_ignore,
                    );

                    app.windows.new_branch_open = true;
                    ui.close();
                }

                if ui.button("  New Tag").clicked() {
                    info!("ui_click_menu_new_tag_open");
                    app.close_all_windows();

                    let new_tag_ignore = app
                        .app_context
                        .session_manager
                        .get_new_tag_ignores()
                        .unwrap_or_default();
                    let new_tag = app
                        .app_context
                        .session_manager
                        .get_new_tag_name()
                        .unwrap_or_default();
                    let new_tag_push = app.app_context.session_manager.get_new_tag_push();

                    app.windows.new_tag.update_settings(
                        &app.app_context.session_manager.project_path,
                        &app.app_context.session_manager.config_file,
                        &app.app_context.repo_manager.mgit_config,
                        new_tag,
                        new_tag_push,
                        &new_tag_ignore,
                    );

                    app.windows.new_tag_open = true;
                    ui.close();
                }

                ui.separator();
                if ui.button("  Clean").clicked() {
                    info!("ui_click_menu_clean_open_dialog");
                    app.close_all_windows();
                    app.windows.clean_dialog_open = true;
                    ui.close();
                }
            });

            ui.menu_button("Settings", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                if ui.button("  Options").clicked() {
                    info!("ui_click_menu_options_open");
                    app.close_all_windows();
                    app.windows.options_open = true;
                    ui.close();
                }
            });

            ui.menu_button("Help", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                if ui.button("  Open Log").clicked() {
                    info!("ui_click_menu_open_log");
                    let log_path = crate::utils::logger::log_dir();
                    #[cfg(target_os = "windows")]
                    std::process::Command::new("explorer")
                        .arg(log_path)
                        .spawn()
                        .ok();

                    #[cfg(target_os = "macos")]
                    std::process::Command::new("open")
                        .arg(log_path)
                        .spawn()
                        .ok();

                    #[cfg(target_os = "linux")]
                    std::process::Command::new("xdg-open")
                        .arg(log_path)
                        .spawn()
                        .ok();

                    ui.close();
                }

                if ui.button("  About").clicked() {
                    info!("ui_click_menu_about_open");
                    app.close_all_windows();
                    app.windows.about_open = true;
                    ui.close();
                }
            });
        });
    }
}
