use eframe::egui;
use std::path::Path;

use mgit::core::git;
use mgit::core::repo::TomlRepo;
use mgit::utils::path::PathExtension;
use mgit::utils::StyleMessage;

use crate::editor::misc::{open_in_file_explorer, open_repo_in_fork};
use crate::editor::ops::{RepoState, StateType};
use crate::editor::Editor;
use crate::toml_settings::cmp_toml_repo;
use crate::utils::defines::{hex_code, text_color};

impl Editor {
    /// part of app/content_view
    pub(crate) fn repositories_list_panel(&mut self, ui: &mut egui::Ui) {
        let desired_width = ui.ctx().used_size().x - 60.0;

        // scroll area
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([true; 2]);
        scroll_area.show(ui, |ui| {
            ui.vertical(|ui| {
                if let Some(mut toml_repos) = self.toml_config.repos.clone() {
                    // modification flag
                    let mut is_modified = false;

                    toml_repos
                        .iter_mut()
                        .enumerate()
                        .for_each(|(idx, toml_repo)| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    ui.set_min_width(desired_width);
                                    ui.horizontal(|ui| {
                                        ui.set_enabled(!self.repo_states[idx].disable_by_label);
                                        // show check box for sync ignore
                                        // save ignore

                                        let checked = if self.repo_states[idx].disable_by_label {
                                            &mut false
                                        } else {
                                            &mut self.repo_states[idx].no_ignore
                                        };

                                        if ui.checkbox(checked, "").changed() {
                                            if let Some(rel_path) = &toml_repo.local {
                                                self.save_ignore(
                                                    rel_path.display_path(),
                                                    !self.repo_states[idx].no_ignore,
                                                );
                                            }
                                        };

                                        // letf panel - repository remote config
                                        self.repository_remote_config_panel(
                                            ui,
                                            toml_repo,
                                            idx,
                                            desired_width * 0.5,
                                        );
                                        // save modification to toml_repo
                                        if cmp_toml_repo(
                                            &self.toml_config.repos.as_ref().unwrap()[idx],
                                            toml_repo,
                                        ) {
                                            is_modified = true;
                                            self.toml_config.repos.as_mut().unwrap()[idx] =
                                                toml_repo.clone();
                                        }

                                        // right panel - repository state
                                        let repo_state = match idx < self.repo_states.len() {
                                            true => self.repo_states[idx].clone(),
                                            false => RepoState::default(),
                                        };
                                        self.repository_state_panel(
                                            ui,
                                            repo_state,
                                            idx,
                                            desired_width * 0.5,
                                        );
                                    });
                                },
                            );
                            ui.separator();
                        });

                    if is_modified {
                        // serialize .gitrepos
                        let toml_string = self.toml_config.serialize();
                        std::fs::write(Path::new(&self.config_file), toml_string)
                            .expect("Failed to write file .gitrepos!");
                    }
                }
            });
        });
    }

    /// part of app/content_view/repositories_list_panel
    pub(crate) fn repository_remote_config_panel(
        &mut self,
        ui: &mut egui::Ui,
        toml_repo: &mut TomlRepo,
        idx: usize,
        desired_width: f32,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(desired_width);

            // show repository name
            // text format by sync ignore
            let rel_path = toml_repo.local.to_owned().unwrap();
            let rep_display = format!("{} {}", hex_code::REPOSITORY, rel_path.display_path());

            // repository name
            ui.horizontal(|ui| {
                ui.set_row_height(18.0);
                // display name
                let hyperlink_color = ui.visuals().hyperlink_color;

                ui.visuals_mut().hyperlink_color = match self.repo_states[idx].no_ignore {
                    true => text_color::PURPLE,
                    false => text_color::DARK_PURPLE,
                };

                if self.repo_states[idx].disable_by_label {
                    ui.visuals_mut().hyperlink_color = text_color::GRAY;
                }

                let response = ui.add(egui::Link::new(rep_display));
                ui.visuals_mut().hyperlink_color = hyperlink_color;

                if response.clicked() {
                    let full_path = match rel_path.as_ref() {
                        "" | "." | "./" => self.project_path.clone(),
                        _ => format!("{}/{}", &self.project_path, &rel_path),
                    };

                    open_repo_in_fork(&full_path);
                }
                response.on_hover_text_at_pointer("Open in Fork");

                let widget_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().max.x + 5.0, ui.min_rect().min.y),
                    egui::vec2(18.0, 12.0),
                );

                // open in file explorer
                let button_response =
                    ui.put(widget_rect, egui::Button::new(hex_code::LINK_EXTERNAL));
                if button_response.clicked() {
                    let full_path = format!("{}/{}", &self.project_path, &rel_path);
                    open_in_file_explorer(&full_path);
                }
            });

            // show remote reference - commit/tag/branch
            let mut remote_ref = String::new();
            let mut branch_text = String::new();
            let mut tag_text = String::new();
            let mut commit_text = String::new();
            if let Some(branch) = toml_repo.branch.to_owned() {
                branch_text = branch.clone();
                remote_ref = format!("{} {}", hex_code::BRANCH, branch);
            }
            if let Some(tag) = toml_repo.tag.to_owned() {
                tag_text = tag.clone();
                remote_ref = format!("{}  {} {}", remote_ref, hex_code::TAG, tag);
            }
            if let Some(commit) = toml_repo.commit.to_owned() {
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

                // edit button
                let current_pos = [ui.min_rect().min.x + 160.0, ui.min_rect().min.y - 40.0];
                let widget_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().max.x + 5.0, ui.min_rect().min.y),
                    egui::vec2(18.0, 18.0),
                );

                let toggle_response = ui.put(
                    widget_rect,
                    egui::SelectableLabel::new(
                        self.remote_ref_edit_idx == idx as i32,
                        hex_code::EDIT,
                    ),
                );

                if toggle_response.clicked() {
                    self.remote_ref_edit_idx = match self.remote_ref_edit_idx == idx as i32 {
                        true => -1,
                        false => idx as i32,
                    };
                }

                if self.remote_ref_edit_idx == idx as i32 {
                    let full_path = Path::new(&self.project_path).join(&rel_path);
                    let remote_branches = git::get_remote_branches(full_path);

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
                                    // branch
                                    ui.label(format!("  {} branch", hex_code::BRANCH));
                                    //ui.add_sized(label_size, egui::TextEdit::singleline(branch_text));

                                    // combo box to select recent project

                                    egui::ComboBox::new(format!("branch_select_{}", idx), "")
                                        .width(290.0)
                                        .selected_text(branch_text.as_str())
                                        .show_ui(ui, |ui| {
                                            is_combo_box_expand = true;
                                            ui.set_min_width(290.0);
                                            for branch in &remote_branches {
                                                if ui.selectable_label(false, branch).clicked() {
                                                    branch_text = branch.to_owned();
                                                }
                                            }
                                        });

                                    //self.memory().open_popup(popup_id);
                                    ui.end_row();

                                    // tag
                                    ui.label(format!("  {} tag", hex_code::TAG));

                                    ui.add_sized(
                                        label_size,
                                        egui::TextEdit::singleline(&mut tag_text),
                                    );
                                    ui.end_row();

                                    // commit
                                    ui.label(format!("  {} commmit", hex_code::COMMIT));
                                    ui.add_sized(
                                        label_size,
                                        egui::TextEdit::singleline(&mut commit_text),
                                    );
                                    ui.end_row();
                                });

                            ui.add_space(5.0);

                            let pointer = &ui.input().pointer;
                            if let Some(pos) = pointer.interact_pos() {
                                if !is_combo_box_expand
                                    && !ui.min_rect().contains(pos)
                                    && !widget_rect.contains(pos)
                                    && pointer.button_clicked(egui::PointerButton::Primary)
                                {
                                    self.remote_ref_edit_idx = -1;
                                }
                            }
                        });
                }

                toml_repo.branch = match branch_text.is_empty() {
                    true => None,
                    false => Some(branch_text),
                };
                toml_repo.tag = match tag_text.is_empty() {
                    true => None,
                    false => Some(tag_text),
                };
                toml_repo.commit = match commit_text.is_empty() {
                    true => None,
                    false => Some(commit_text),
                };
            });

            if let Some(labels) = &toml_repo.labels {
                let text = format!("{} {}", hex_code::LABEL, labels.join(" "));
                ui.label(text);
            }

            // show remote url
            let url = format!(
                "{} {}",
                hex_code::URL,
                toml_repo.remote.to_owned().unwrap().display_path()
            );
            let job = create_truncate_layout_job(url, text_color::LIGHT_GRAY);
            ui.label(job);
        });
    }

    /// part of app/content_view/repositories_list_panel
    pub(crate) fn repository_state_panel(
        &mut self,
        ui: &mut egui::Ui,
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

                        let job =
                            create_layout_jobs(&self.ops_message_collector.read_ops_message(idx));
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
}

pub fn create_layout_job(text: String, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &text,
        0.0,
        egui::TextFormat {
            color,
            ..Default::default()
        },
    );
    job
}

pub fn create_layout_jobs(style_message: &StyleMessage) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut len = 0;
    for sm in &style_message.0 {
        // 实现一个truncate效果
        if len + sm.content.len() > 100 {
            break;
        }
        job.append(
            &sm.content,
            0.0,
            egui::TextFormat {
                color: text_color::from(sm.style.map_or(&None, |s| &s.foreground)),
                ..Default::default()
            },
        );
        len += sm.content.len();
    }
    job
}

pub fn create_truncate_layout_job(text: String, color: egui::Color32) -> egui::text::LayoutJob {
    egui::text::LayoutJob {
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: egui::TextFormat::simple(egui::FontId::default(), color),
        }],
        wrap: egui::epaint::text::TextWrapping {
            max_rows: 1,
            break_anywhere: true,
            ..Default::default()
        },
        break_on_newline: false,
        halign: egui::Align::RIGHT,
        justify: true,
        text,
        ..Default::default()
    }
}
