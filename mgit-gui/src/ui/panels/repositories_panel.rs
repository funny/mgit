use std::path::Path;
use std::time::Instant;

use eframe::egui;

use mgit::config::RepoConfig;
use mgit::git;
use mgit::utils::path::PathExtension;
use tracing::{debug, info, warn};

use crate::app::context::{RepoState, StateType};
use crate::app::events::{BackendEvent, Event};
use crate::app::GuiApp;
use crate::ui::components::{create_layout_job, create_layout_jobs, create_truncate_layout_job};
use crate::ui::style::{hex_code, text_color};
use crate::utils::system::{open_in_file_explorer, open_repo_in_fork};

pub(crate) struct RepositoriesPanel;

impl RepositoriesPanel {
    pub(crate) fn show(ui: &mut egui::Ui, app: &mut GuiApp) {
        let desired_width = ui.ctx().used_size().x - 60.0;

        {
            let mut total = 0usize;
            let mut unignored = 0usize;
            if let Some(repo_configs) = &app.app_context.repo_manager.mgit_config.repos {
                for (repo_config, state) in repo_configs
                    .iter()
                    .zip(&app.app_context.repo_manager.repo_states)
                {
                    if state.disable_by_label {
                        continue;
                    }
                    if repo_config.local.is_some() {
                        total += 1;
                        if state.no_ignore {
                            unignored += 1;
                        }
                    }
                }
            }
            let mut all_unignored = total > 0 && unignored == total;
            let indeterminate = unignored > 0 && unignored < total;
            let response = ui.add(
                egui::Checkbox::new(&mut all_unignored, " Unignore All / Ignore All")
                    .indeterminate(indeterminate),
            );
            if response.changed() {
                let target_ignore = !all_unignored;
                if let Some(repo_configs) = &app.app_context.repo_manager.mgit_config.repos {
                    for (repo_config, state) in repo_configs
                        .iter()
                        .zip(&app.app_context.repo_manager.repo_states)
                    {
                        if state.disable_by_label {
                            continue;
                        }
                        if let Some(rel_path) = &repo_config.local {
                            app.app_context
                                .session_manager
                                .project_settings
                                .save_ignore(rel_path.display_path(), target_ignore);
                        }
                    }
                    app.app_context.session_manager.save_project_settings();
                    for (_, state) in repo_configs
                        .iter()
                        .zip(&mut app.app_context.repo_manager.repo_states)
                    {
                        if state.disable_by_label {
                            continue;
                        }
                        state.no_ignore = !target_ignore;
                    }
                }
            }
        }

        let scroll_area = egui::ScrollArea::vertical().auto_shrink([true; 2]);
        scroll_area.show(ui, |ui| {
            ui.vertical(|ui| {
                let Some(mut repo_configs) = app.app_context.repo_manager.mgit_config.repos.take()
                else {
                    return;
                };

                let mut is_modified = false;

                for (idx, repo_config) in repo_configs.iter_mut().enumerate() {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        ui.set_min_width(desired_width);
                        ui.horizontal(|ui| {
                            let checked =
                                if app.app_context.repo_manager.repo_states[idx].disable_by_label {
                                    &mut false
                                } else {
                                    &mut app.app_context.repo_manager.repo_states[idx].no_ignore
                                };

                            if ui.checkbox(checked, "").on_hover_text("ignore").changed() {
                                if let Some(rel_path) = &repo_config.local {
                                    info!(
                                        repo_rel_path = rel_path.display_path(),
                                        ignore = !app.app_context.repo_manager.repo_states[idx]
                                            .no_ignore,
                                        "ui_toggle_repo_ignore"
                                    );
                                    app.app_context.session_manager.save_ignore(
                                        rel_path.display_path(),
                                        !app.app_context.repo_manager.repo_states[idx].no_ignore,
                                    );
                                }
                            };

                            ui.add_enabled_ui(
                                !app.app_context.repo_manager.repo_states[idx].disable_by_label,
                                |ui| {
                                    is_modified |= Self::repository_remote_config_panel(
                                        ui,
                                        app,
                                        repo_config,
                                        idx,
                                        desired_width * 0.5,
                                    );

                                    let repo_state = match idx
                                        < app.app_context.repo_manager.repo_states.len()
                                    {
                                        true => {
                                            app.app_context.repo_manager.repo_states[idx].clone()
                                        }
                                        false => RepoState::default(),
                                    };
                                    Self::repository_state_panel(
                                        ui,
                                        app,
                                        repo_state,
                                        idx,
                                        desired_width * 0.5,
                                    );
                                },
                            );
                        });
                    });
                    ui.separator();
                }

                app.app_context.repo_manager.mgit_config.repos = Some(repo_configs);
                if is_modified {
                    let toml_string = app.app_context.repo_manager.mgit_config.serialize();
                    let config_file = app.app_context.session_manager.config_file.clone();
                    app.app_context.repo_manager.schedule_config_save(
                        app.app_context.next_run_id(),
                        config_file,
                        toml_string,
                    );
                }
            });
        });
    }

    fn repository_remote_config_panel(
        ui: &mut egui::Ui,
        app: &mut GuiApp,
        repo_config: &mut RepoConfig,
        idx: usize,
        desired_width: f32,
    ) -> bool {
        let old_branch = repo_config.branch.clone();
        let old_tag = repo_config.tag.clone();
        let old_commit = repo_config.commit.clone();

        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(desired_width);

            let rel_path = repo_config.local.to_owned().unwrap();
            let rep_display = format!("{} {}", hex_code::REPOSITORY, rel_path.display_path());

            ui.horizontal(|ui| {
                ui.set_row_height(18.0);
                let hyperlink_color = ui.visuals().hyperlink_color;

                ui.visuals_mut().hyperlink_color =
                    match app.app_context.repo_manager.repo_states[idx].no_ignore {
                        true => text_color::PURPLE,
                        false => text_color::DARK_PURPLE,
                    };

                if app.app_context.repo_manager.repo_states[idx].disable_by_label {
                    ui.visuals_mut().hyperlink_color = text_color::GRAY;
                }

                let response = ui
                    .add(egui::Link::new(rep_display))
                    .on_hover_text("LMB open folder, RMB open in fork");
                ui.visuals_mut().hyperlink_color = hyperlink_color;

                if response.clicked() {
                    let full_path = match rel_path.as_ref() {
                        "" | "." | "./" => app.app_context.session_manager.project_path.clone(),
                        _ => format!(
                            "{}/{}",
                            &app.app_context.session_manager.project_path, &rel_path
                        ),
                    };
                    info!(
                        repo_rel_path = rel_path.display_path(),
                        "ui_open_repo_in_explorer"
                    );
                    open_in_file_explorer(&full_path);
                }

                if response.secondary_clicked() {
                    let full_path = match rel_path.as_ref() {
                        "" | "." | "./" => app.app_context.session_manager.project_path.clone(),
                        _ => format!(
                            "{}/{}",
                            &app.app_context.session_manager.project_path, &rel_path
                        ),
                    };
                    info!(
                        repo_rel_path = rel_path.display_path(),
                        "ui_open_repo_in_fork"
                    );
                    open_repo_in_fork(&full_path);
                }
            });

            ui.add_space(4.0);

            {
                let mut remote_ref = String::new();
                let mut branch_text = String::new();
                let mut tag_text = String::new();
                let mut commit_text = String::new();
                if let Some(branch) = repo_config.branch.to_owned() {
                    branch_text = branch.clone();
                    remote_ref = format!("{} {}", hex_code::BRANCH, branch);
                }
                if let Some(tag) = repo_config.tag.to_owned() {
                    tag_text = tag.clone();
                    remote_ref = format!("{}  {} {}", remote_ref, hex_code::TAG, tag);
                }
                if let Some(commit) = repo_config.commit.to_owned() {
                    commit_text = commit.clone();

                    let commit = match commit.len() < 7 {
                        true => &commit,
                        false => &commit[0..7],
                    };
                    remote_ref = format!("{}  {} {}", remote_ref, hex_code::COMMIT, commit);
                }
                let job = create_truncate_layout_job(remote_ref, text_color::GRAY);

                ui.horizontal(|ui| {
                    ui.label(job);

                    let current_pos = [ui.min_rect().min.x + 160.0, ui.min_rect().min.y - 40.0];
                    let widget_rect = egui::Rect::from_min_size(
                        egui::pos2(ui.min_rect().max.x + 5.0, ui.min_rect().min.y),
                        egui::vec2(18.0, 18.0),
                    );

                    let toggle_response = ui.put(
                        widget_rect,
                        egui::Button::new(hex_code::EDIT).selected(
                            app.app_context.repo_manager.remote_ref_edit_idx == idx as i32,
                        ),
                    );

                    if toggle_response.clicked() {
                        app.app_context.repo_manager.remote_ref_edit_idx =
                            match app.app_context.repo_manager.remote_ref_edit_idx == idx as i32 {
                                true => -1,
                                false => idx as i32,
                            };

                        if app.app_context.repo_manager.remote_ref_edit_idx == idx as i32 {
                            info!(
                                repo_rel_path = rel_path.display_path(),
                                "ui_open_remote_ref_editor"
                            );
                            Self::request_remote_branches(app, &rel_path);
                        } else {
                            info!(
                                repo_rel_path = rel_path.display_path(),
                                "ui_close_remote_ref_editor"
                            );
                        }
                    }

                    if app.app_context.repo_manager.remote_ref_edit_idx == idx as i32 {
                        if !app
                            .app_context
                            .repo_manager
                            .remote_branches_cache
                            .contains_key(&rel_path)
                        {
                            Self::request_remote_branches(app, &rel_path);
                        }
                        let (is_loading, remote_branches, load_error) = match app
                            .app_context
                            .repo_manager
                            .remote_branches_cache
                            .get(&rel_path)
                        {
                            Some(entry) => {
                                (entry.loading, entry.branches.clone(), entry.error.clone())
                            }
                            None => (false, Vec::new(), None::<String>),
                        };

                        egui::Window::new(format!("{} config", &rel_path))
                            .fixed_pos(current_pos)
                            .resizable(false)
                            .collapsible(false)
                            .title_bar(false)
                            .open(&mut true)
                            .show(ui.ctx(), |ui| {
                                let mut is_combo_box_expand = false;

                                ui.add_space(5.0);

                                egui::Grid::new(format!("repo_editing_panel_{}", idx))
                                    .striped(false)
                                    .num_columns(3)
                                    .min_col_width(60.0)
                                    .show(ui, |ui| {
                                        ui.set_width(410.0);
                                        let label_size = [300.0, 20.0];
                                        ui.label(format!("  {} branch", hex_code::BRANCH));

                                        egui::ComboBox::new(format!("branch_select_{}", idx), "")
                                            .width(290.0)
                                            .selected_text(branch_text.as_str())
                                            .show_ui(ui, |ui| {
                                                is_combo_box_expand = true;
                                                ui.set_min_width(290.0);
                                                for branch in &remote_branches {
                                                    if ui.selectable_label(false, branch).clicked()
                                                    {
                                                        info!(
                                                            repo_rel_path = rel_path.display_path(),
                                                            branch = branch.as_str(),
                                                            "ui_select_remote_branch"
                                                        );
                                                        branch_text = branch.to_owned();
                                                    }
                                                }
                                            });
                                        if is_loading {
                                            ui.add(egui::Spinner::new());
                                        } else if ui.button("refresh").clicked() {
                                            info!(
                                                repo_rel_path = rel_path.display_path(),
                                                "ui_refresh_remote_branches"
                                            );
                                            if let Some(entry) = app
                                                .app_context
                                                .repo_manager
                                                .remote_branches_cache
                                                .get_mut(&rel_path)
                                            {
                                                entry.loading = false;
                                                entry.branches.clear();
                                                entry.error = None;
                                            }
                                            Self::request_remote_branches(app, &rel_path);
                                        }
                                        if let Some(error) = load_error.as_ref() {
                                            ui.colored_label(
                                                ui.visuals().error_fg_color,
                                                format!("load branches failed: {}", error),
                                            );
                                            if ui.button("retry").clicked() {
                                                info!(
                                                    repo_rel_path = rel_path.display_path(),
                                                    "ui_retry_remote_branches"
                                                );
                                                if let Some(entry) = app
                                                    .app_context
                                                    .repo_manager
                                                    .remote_branches_cache
                                                    .get_mut(&rel_path)
                                                {
                                                    entry.loading = false;
                                                    entry.branches.clear();
                                                    entry.error = None;
                                                }
                                                Self::request_remote_branches(app, &rel_path);
                                            }
                                        }

                                        ui.end_row();

                                        ui.label(format!("  {} tag", hex_code::TAG));

                                        ui.add_sized(
                                            label_size,
                                            egui::TextEdit::singleline(&mut tag_text),
                                        );
                                        ui.end_row();

                                        ui.label(format!("  {} commmit", hex_code::COMMIT));
                                        ui.add_sized(
                                            label_size,
                                            egui::TextEdit::singleline(&mut commit_text),
                                        );
                                        ui.end_row();
                                    });

                                ui.add_space(5.0);

                                let pointer = &ui.input(|i| i.pointer.clone());
                                if let Some(pos) = pointer.interact_pos() {
                                    if !is_combo_box_expand
                                        && !ui.min_rect().contains(pos)
                                        && !widget_rect.contains(pos)
                                        && pointer.button_clicked(egui::PointerButton::Primary)
                                    {
                                        app.app_context.repo_manager.remote_ref_edit_idx = -1;
                                    }
                                }
                            });
                    }

                    repo_config.branch = match branch_text.is_empty() {
                        true => None,
                        false => Some(branch_text),
                    };
                    repo_config.tag = match tag_text.is_empty() {
                        true => None,
                        false => Some(tag_text),
                    };
                    repo_config.commit = match commit_text.is_empty() {
                        true => None,
                        false => Some(commit_text),
                    };
                });

                if let Some(labels) = &repo_config.labels {
                    let text = format!("{} {}", hex_code::LABEL, labels.join(" "));
                    ui.label(text);
                }

                let url = format!(
                    "{} {}",
                    hex_code::URL,
                    repo_config.remote.to_owned().unwrap().display_path()
                );
                let job = create_truncate_layout_job(url, text_color::LIGHT_GRAY);
                ui.label(job);
            }
        });

        let changed = repo_config.branch != old_branch
            || repo_config.tag != old_tag
            || repo_config.commit != old_commit;
        if changed {
            debug!(
                repo_rel_path = repo_config.local.as_deref().unwrap_or(""),
                branch = repo_config.branch.as_deref().unwrap_or(""),
                tag = repo_config.tag.as_deref().unwrap_or(""),
                commit = repo_config.commit.as_deref().unwrap_or(""),
                "ui_edit_repo_remote_ref_changed"
            );
        }
        changed
    }

    fn repository_state_panel(
        ui: &mut egui::Ui,
        app: &mut GuiApp,
        repo_state: RepoState,
        idx: usize,
        desired_width: f32,
    ) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.set_width(desired_width);

            if repo_state.disable_by_label {
                let job = create_layout_job(
                    format!("{} Disconnected", hex_code::DISCONNECTED),
                    text_color::GRAY,
                );
                ui.label(job);
                ui.add_space(4.0);
                return;
            }
            if repo_state.err_msg.is_empty() {
                // show states
                match repo_state.track_state {
                    // show disconnected
                    StateType::Disconnected => {
                        let job = create_layout_job(
                            format!("{} Disconnected", hex_code::DISCONNECTED),
                            text_color::GRAY,
                        );
                        ui.label(job);
                        ui.add_space(4.0);
                    }
                    // show updating
                    StateType::Updating => {
                        let job = create_layout_job(
                            format!("{} Updating...", hex_code::UPDATING),
                            text_color::GREEN,
                        );
                        ui.horizontal(|ui| {
                            ui.label(job);
                            ui.add(egui::widgets::Spinner::new());
                        });

                        let job = create_layout_jobs(
                            &app.app_context
                                .repo_manager
                                .ops_message_collector
                                .read_ops_message(idx),
                        );
                        ui.label(job);

                        ui.add_space(4.0);
                    }
                    // show Warning
                    StateType::Warning => {
                        ui.horizontal(|ui| {
                            let job = create_layout_job(
                                format!("{} Warning", hex_code::WARNING),
                                text_color::YELLOW,
                            );
                            ui.label(job);

                            for (idx, tag) in repo_state.tags.iter().enumerate() {
                                let job = create_layout_job(
                                    format!("{} {}", hex_code::TAG, tag),
                                    text_color::LIGHT_GRAY,
                                );
                                ui.label(job);

                                if idx >= 3 {
                                    break;
                                }
                            }
                        });
                        ui.add_space(4.0);

                        // show untracked
                        let mut job = create_layout_job(
                            format!("{} {}", hex_code::BRANCH, &repo_state.current_branch),
                            text_color::BLUE,
                        );

                        job.append(" ", 0.0, egui::TextFormat::default());
                        job.append(
                            &repo_state.tracking_branch,
                            0.0,
                            egui::TextFormat {
                                color: text_color::YELLOW,
                                ..Default::default()
                            },
                        );
                        ui.label(job);
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            // show normal
                            let job = create_layout_job(
                                format!("{} Normal", hex_code::NORMAL),
                                text_color::GREEN,
                            );
                            ui.label(job);

                            for (idx, tag) in repo_state.tags.iter().enumerate() {
                                let job = create_layout_job(
                                    format!("{} {}", hex_code::TAG, tag),
                                    text_color::LIGHT_GRAY,
                                );
                                ui.label(job);

                                if idx >= 3 {
                                    break;
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // show track
                        let track_str = format!(
                            "{} {} {} {}",
                            hex_code::BRANCH,
                            &repo_state.current_branch,
                            hex_code::ARROW_RIGHT_BOLD,
                            &repo_state.tracking_branch
                        );
                        let job = create_truncate_layout_job(track_str, text_color::BLUE);
                        ui.label(job);

                        // show commit
                        // Normal
                        if repo_state.cmp_state == StateType::Normal {
                            let job = create_truncate_layout_job(
                                format!("{} {}", hex_code::COMMIT, &repo_state.cmp_obj),
                                text_color::GRAY,
                            );
                            ui.label(job);
                        } else if repo_state.cmp_state == StateType::Warning {
                            // Warning
                            let mut job = egui::text::LayoutJob::default();
                            job.append(
                                &format!("{} ", hex_code::COMMIT),
                                0.0,
                                egui::TextFormat::default(),
                            );
                            if !repo_state.cmp_commit.is_empty() {
                                job.append(
                                    &repo_state.cmp_commit,
                                    0.0,
                                    egui::TextFormat {
                                        color: text_color::YELLOW,
                                        ..Default::default()
                                    },
                                );
                                job.append(" ", 0.0, egui::TextFormat::default());
                            }
                            job.append(
                                &repo_state.cmp_changes,
                                0.0,
                                egui::TextFormat {
                                    color: text_color::RED,
                                    ..Default::default()
                                },
                            );
                            ui.label(job);
                        } else {
                            // Error
                            let job = create_truncate_layout_job(
                                format!("{} {}", hex_code::COMMIT, &repo_state.cmp_obj),
                                text_color::RED,
                            );
                            ui.label(job);
                        }
                    }
                }
            }
            // show error
            else {
                let job = create_layout_job(format!("{} Error", hex_code::ERROR), text_color::RED);
                ui.label(job);
                ui.add_space(4.0);

                let job = create_truncate_layout_job(
                    format!("{} {}", hex_code::ISSUE, &repo_state.err_msg),
                    text_color::RED,
                );
                ui.label(job);
            }
        });
    }

    fn request_remote_branches(app: &mut GuiApp, rel_path: &str) {
        let entry = app
            .app_context
            .repo_manager
            .remote_branches_cache
            .entry(rel_path.to_string())
            .or_default();
        if entry.loading || entry.error.is_some() || !entry.branches.is_empty() {
            return;
        }

        entry.loading = true;
        entry.error = None;
        entry.branches.clear();

        let run_id = app.app_context.next_run_id();
        let sender = app.app_context.event_tx.clone();
        let project_path = app.app_context.session_manager.project_path.clone();
        let rel_path = rel_path.to_string();
        std::thread::spawn(move || {
            let started_at = Instant::now();
            info!(
                run_id,
                repo_rel_path = rel_path.as_str(),
                "remote_branches_load_start"
            );
            let full_path = Path::new(&project_path).join(&rel_path);
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    warn!(
                        run_id,
                        repo_rel_path = rel_path.as_str(),
                        error = %e,
                        "tokio_runtime_create_failed"
                    );
                    return;
                }
            };
            match rt.block_on(git::get_remote_branches(full_path)) {
                Ok(branches) => {
                    info!(
                        run_id,
                        repo_rel_path = rel_path.as_str(),
                        branch_count = branches.len(),
                        duration_ms = started_at.elapsed().as_millis(),
                        "remote_branches_load_ok"
                    );
                    let _ = sender.send(Event::Backend(BackendEvent::RemoteBranchesLoaded {
                        run_id,
                        repo_rel_path: rel_path,
                        branches,
                    }));
                }
                Err(e) => {
                    warn!(
                        run_id,
                        repo_rel_path = rel_path.as_str(),
                        duration_ms = started_at.elapsed().as_millis(),
                        error = %e,
                        "remote_branches_load_failed"
                    );
                    let _ = sender.send(Event::Backend(BackendEvent::RemoteBranchesFailed {
                        run_id,
                        repo_rel_path: rel_path,
                        error: e.to_string(),
                    }));
                }
            }
        });
    }
}
