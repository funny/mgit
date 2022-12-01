use super::*;
use eframe::egui;
use mgit::commands::{
    cmp_local_remote, display_path, get_current_branch, get_tracking_branch, is_repository,
    load_config, norm_path,
};
use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// main app ui update
impl eframe::App for App {
    // save app state
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        // top view
        self.top_view(ctx);

        // content view
        self.content_view(ctx);

        // show windows
        self.handle_windows(ctx, eframe);

        // handle channel recv
        self.handle_channel_recv();
    }
}

// ========================================
// data handle for app
// ========================================
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // restore app state
        #[cfg(feature = "persistence")]
        if let Some(storage) = _cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        setup_custom_fonts(&cc.egui_ctx);
        configure_text_styles(&cc.egui_ctx);

        let mut app = App::default();
        app.load_setting();
        app.execute_cmd(CommandType::Refresh);
        app
    }

    // startup with arg: mgit-gui <path>
    fn get_path_from_env_args(&self) -> Option<String> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            let path = PathBuf::from(args[1].clone());
            if let Ok(path) = std::fs::canonicalize(path) {
                let path = norm_path(&format!("{}", path.display()));

                let norm_path = path.replace("//?/", "");
                return Some(norm_path);
            }
        }
        None
    }

    fn load_setting(&mut self) {
        self.toml_user_settings = TomlUserSettings::load();
        self.load_recent_projects();

        // if app startup with args including project path, use the path
        if let Some(startup_project) = self.get_path_from_env_args() {
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

        // restore options setting
        self.options_window
            .load_option_from_settings(&self.toml_user_settings, &self.get_snapshot_ignore());
    }

    fn load_config(&mut self) {
        let config_file = PathBuf::from(&self.config_file);
        if config_file.is_file() {
            if let Some(toml_config) = load_config(&config_file) {
                self.toml_config = toml_config;

                // init repo states and sync ignore
                if let Some(toml_repos) = &self.toml_config.repos {
                    for toml_repo in toml_repos {
                        // get ignore state
                        let do_ignore = if let Some(ignore) = &self.get_ignore() {
                            let ignore_paths: Vec<String> =
                                ignore.split("\n").map(|s| s.trim().to_string()).collect();
                            let rel_path = toml_repo
                                .local
                                .to_owned()
                                .unwrap_or(String::from("invalid"));
                            ignore_paths.contains(&display_path(&rel_path))
                        } else {
                            false
                        };

                        // init repo state
                        self.repo_states.push(RepoState {
                            no_ignore: !do_ignore,
                            ..RepoState::default()
                        });
                    }
                }
            }
        }
    }

    fn clear_toml_config(&mut self) {
        self.repo_states = Vec::new();
        self.toml_config = TomlConfig::default();
    }

    fn reset_repo_state(&mut self, state_type: StateType) {
        for repo_state in &mut self.repo_states {
            *repo_state = RepoState {
                track_state: state_type.clone(),
                cmp_state: state_type.clone(),
                no_ignore: repo_state.no_ignore,
                ..RepoState::default()
            };
        }
    }

    fn get_repo_states(&mut self) {
        // get repository state
        if let Some(repos) = &self.toml_config.repos.clone() {
            let project_path = self.project_path.clone();
            let default_branch = self.toml_config.default_branch.clone();
            get_repo_states_thread(
                repos.to_owned(),
                project_path,
                default_branch,
                self.send.clone(),
            )
        }
    }

    fn handle_channel_recv(&mut self) {
        // as callback after execute command
        if let Ok((command_type, (idx, repo_state))) = self.recv.try_recv() {
            if command_type == CommandType::None {
                if idx < self.repo_states.len() {
                    self.repo_states[idx] = RepoState {
                        no_ignore: self.repo_states[idx].no_ignore,
                        ..repo_state
                    };
                }
            } else {
                self.load_config();
                self.reset_repo_state(StateType::Updating);
                self.get_repo_states();
            }
        }
    }

    fn execute_cmd(&mut self, command_type: CommandType) {
        // to show in ui
        if self.config_file.is_empty() {
            self.config_file = format!("{}/.gitrepos", &self.project_path);
        }

        match command_type {
            CommandType::Init => {
                self.config_file = format!("{}/.gitrepos", &self.project_path);
                // option --force
                let options = match self.toml_user_settings.init_force.unwrap_or(true) {
                    true => String::from("--force"),
                    false => String::new(),
                };

                self.clear_toml_config();
                execute_cmd_with_send(
                    "init",
                    &self.project_path,
                    &options,
                    CommandType::Init,
                    self.send.clone(),
                );
            }
            CommandType::Snapshot => {
                // option --config
                let mut options = match self.config_file.is_empty() {
                    true => String::new(),
                    false => format!("--config \"{}\"", &self.config_file),
                };
                // option --branch
                if self.toml_user_settings.snapshot_branch.unwrap_or(true) {
                    options = format!("{} --branch", options);
                }
                // option --force
                if self.toml_user_settings.snapshot_force.unwrap_or(true) {
                    options = format!("{} --force", options);
                }
                // option --ignore
                if let Some(ignore_paths) = &self.get_snapshot_ignore() {
                    let ignore_paths: Vec<&str> = ignore_paths.split("\n").collect();
                    for ignore_path in ignore_paths {
                        if !ignore_path.is_empty() {
                            options = format!("{} --ignore \"{}\"", options, ignore_path.trim());
                        }
                    }
                }

                self.clear_toml_config();
                execute_cmd_with_send(
                    "snapshot",
                    &self.project_path,
                    &options,
                    CommandType::Snapshot,
                    self.send.clone(),
                );
            }
            CommandType::Fetch => {
                // option --config
                let mut options = match self.config_file.is_empty() {
                    true => String::new(),
                    false => format!("--config \"{}\"", &self.config_file),
                };
                // option --thread <num>
                if let Some(thread) = self.toml_user_settings.sync_thread {
                    options = format!("{} --thread {}", options, thread);
                }
                // option --ignore
                if let Some(ignore_paths) = &self.get_ignore() {
                    let ignore_paths: Vec<&str> = ignore_paths.split("\n").collect();
                    for ignore_path in ignore_paths {
                        if !ignore_path.is_empty() {
                            options = format!("{} --ignore \"{}\"", options, ignore_path.trim());
                        }
                    }
                }
                // option --silent
                options = format!("{} --silent", options);

                self.reset_repo_state(StateType::Updating);
                execute_cmd_with_send(
                    "fetch",
                    &self.project_path,
                    &options,
                    CommandType::Fetch,
                    self.send.clone(),
                );
            }
            CommandType::Sync | CommandType::SyncHard => {
                // option --config
                let mut options = match self.config_file.is_empty() {
                    true => String::new(),
                    false => format!("--config \"{}\"", &self.config_file),
                };
                // check if command_type is CommandType::SyncHard
                let sync_type = match command_type == CommandType::SyncHard {
                    true => SyncType::Hard,
                    false => self.toml_user_settings.sync_type.unwrap_or(SyncType::Stash),
                };
                // option none or --stash or --hard
                match sync_type {
                    SyncType::Normal => {}
                    SyncType::Stash => options = format!("{} --stash", options),
                    SyncType::Hard => options = format!("{} --hard", options),
                };
                // option --no-checkout
                if self.toml_user_settings.sync_no_checkout.unwrap_or(false) {
                    options = format!("{} --no-checkout", options);
                }
                // option --no-track
                if self.toml_user_settings.sync_no_track.unwrap_or(false) {
                    options = format!("{} --no-track", options);
                }
                // option --thread <num>
                if let Some(thread) = self.toml_user_settings.sync_thread {
                    options = format!("{} --thread {}", options, thread);
                }
                // option --ignore
                if let Some(ignore_paths) = &self.get_ignore() {
                    let ignore_paths: Vec<&str> = ignore_paths.split("\n").collect();
                    for ignore_path in ignore_paths {
                        if !ignore_path.is_empty() {
                            options = format!("{} --ignore \"{}\"", options, ignore_path.trim());
                        }
                    }
                }

                // option --silent
                options = format!("{} --silent", options);

                self.reset_repo_state(StateType::Updating);
                execute_cmd_with_send(
                    "sync",
                    &self.project_path,
                    &options,
                    CommandType::Sync,
                    self.send.clone(),
                );
            }
            CommandType::Track => {
                // option --config
                let mut options = match self.config_file.is_empty() {
                    true => String::new(),
                    false => format!("--config \"{}\"", &self.config_file),
                };
                // option --ignore
                if let Some(ignore_paths) = &self.get_ignore() {
                    let ignore_paths: Vec<&str> = ignore_paths.split("\n").collect();
                    for ignore_path in ignore_paths {
                        if !ignore_path.is_empty() {
                            options = format!("{} --ignore \"{}\"", options, ignore_path.trim());
                        }
                    }
                }

                self.reset_repo_state(StateType::Updating);
                execute_cmd_with_send(
                    "track",
                    &self.project_path,
                    &options,
                    CommandType::Track,
                    self.send.clone(),
                );
            }
            CommandType::Clean => {
                // option --config
                let options = match self.config_file.is_empty() {
                    true => String::new(),
                    false => format!("--config \"{}\"", &self.config_file),
                };

                self.reset_repo_state(StateType::Updating);
                execute_cmd_with_send(
                    "clean",
                    &self.project_path,
                    &options,
                    CommandType::Clean,
                    self.send.clone(),
                );
            }
            CommandType::Refresh => {
                self.clear_toml_config();
                self.load_config();
                self.reset_repo_state(StateType::Updating);
                self.get_repo_states();
            }
            CommandType::None => {}
        }
    }
}

// ========================================
// ui design for app
// ========================================
impl App {
    /// part of app
    fn handle_windows(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        // show about window
        self.about_window.show(ctx, eframe, &mut self.about_is_open);

        // show options window
        self.options_window
            .show(ctx, eframe, &mut self.options_is_open);
        if self.options_is_open {
            self.toml_user_settings.save_options(&self.options_window);
            self.save_snapshot_ignore();
        }

        // show clean dialog
        self.clean_dialog.show(ctx, eframe, &mut self.clean_is_open);
        if self.clean_dialog.is_ok() {
            self.execute_cmd(CommandType::Clean);
        }

        // show sync hard dialog
        self.sync_hard_dialog
            .show(ctx, eframe, &mut self.sync_hard_is_open);
        if self.sync_hard_dialog.is_ok() {
            self.execute_cmd(CommandType::SyncHard);
        }
    }

    fn close_all_windows(&mut self) {
        self.about_is_open = false;
        self.options_is_open = false;
        self.clean_is_open = false;
        self.sync_hard_is_open = false;
    }

    /// quick bar panel of app
    fn top_view(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mgit_gui_top_bar").show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 5.0);

            // menu bar
            self.menu_bar(ui);

            // quick bar
            self.quick_bar(ui);
            ui.add_space(2.0);
        });
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            // Commands menu
            ui.menu_button("Commands", |ui| {
                ui.set_min_width(MENU_BOX_WIDTH);
                // init button
                if ui.button("  Init").clicked() {
                    self.execute_cmd(CommandType::Init);
                    ui.close_menu();
                }

                if ui.button("  Snapshot").clicked() {
                    self.execute_cmd(CommandType::Snapshot);
                    ui.close_menu();
                }

                // fetch button
                if ui.button("  Fetch").clicked() {
                    self.execute_cmd(CommandType::Fetch);
                    ui.close_menu();
                }

                // sync button
                if ui.button("  Sync").clicked() {
                    self.execute_cmd(CommandType::Sync);
                    ui.close_menu();
                }

                // track button
                if ui.button("  Track").clicked() {
                    self.execute_cmd(CommandType::Track);
                    ui.close_menu();
                }

                // clean button
                if ui.button("  Clean").clicked() {
                    self.close_all_windows();
                    self.clean_is_open = true;
                    ui.close_menu();
                }

                // refresh button
                if ui.button("  Refresh").clicked() {
                    self.execute_cmd(CommandType::Refresh);
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

    fn quick_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            let button_size = [96.0, 36.0];
            // fetch button
            let fetch_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("  {}\nFetch", hex_code::FETCH)),
            );
            if fetch_button_response.clicked() {
                self.execute_cmd(CommandType::Fetch);
            }

            // sync button
            let sync_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!(" {}\nSync", hex_code::SYNC)),
            );
            if sync_button_response.clicked() {
                self.execute_cmd(CommandType::Sync);
            }

            // sync hard button
            let sync_hard_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("     {}\nSync (Hard)", hex_code::SYNC)),
            );
            if sync_hard_button_response.clicked() {
                self.close_all_windows();
                self.sync_hard_is_open = true;
            }

            // refress button
            let refresh_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("   {}\nRefresh", hex_code::REFRESH)),
            );
            if refresh_button_response.clicked() {
                self.execute_cmd(CommandType::Refresh);
            }
        });
    }

    /// content_view of app
    fn content_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.set_min_width(DEFAULT_WIDTH);
                ui.heading("Configuration");

                // configuration detail
                self.configuration_panel(ui);

                ui.separator();

                // repositories list detail
                let repos_count = match &self.toml_config.repos {
                    Some(toml_repos) => toml_repos.len(),
                    _ => 0,
                };

                ui.heading(format!("Repositories ({})", repos_count));
                self.repositories_list_panel(ui);
            });
        });
    }

    /// part of app/content_view
    fn configuration_panel(&mut self, ui: &mut egui::Ui) {
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
                        self.project_path = norm_path(&path.display().to_string());
                        is_project_changed = true;
                    }
                }

                // button to open in file in explorer
                if ui
                    .add_sized([18.0, 18.0], egui::Button::new(hex_code::LINK_EXTERNAL))
                    .clicked()
                {
                    if Path::new(&self.project_path).is_dir() {
                        open_in_file_explorer(self.project_path.clone());
                    }
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
                    self.project_path = norm_path(&self.project_path);

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
                        self.config_file = norm_path(&path.display().to_string());
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
                    self.config_file = norm_path(&self.config_file);
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

                    self.execute_cmd(CommandType::Refresh);
                }
                ui.end_row();
            });
    }

    /// part of app/content_view
    fn repositories_list_panel(&mut self, ui: &mut egui::Ui) {
        let desired_width = ui.ctx().used_size().x - 60.0;

        // scroll area
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([true; 2]);
        scroll_area.show(ui, |ui| {
            ui.vertical(|ui| {
                if let Some(toml_repos) = self.toml_config.repos.clone() {
                    // modification flag
                    let mut is_modified = false;

                    for idx in 0..toml_repos.len() {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.set_min_width(desired_width);
                            ui.horizontal(|ui| {
                                let mut toml_repo = toml_repos[idx].clone();

                                // show check box for sync ignore
                                // save ignore
                                if ui
                                    .checkbox(&mut self.repo_states[idx].no_ignore, "")
                                    .changed()
                                {
                                    if let Some(rel_path) = &toml_repo.local {
                                        self.save_ignore(
                                            display_path(rel_path),
                                            !self.repo_states[idx].no_ignore,
                                        );
                                    }
                                };

                                // letf panel - repository remote config
                                self.repository_remote_config_panel(
                                    ui,
                                    &mut toml_repo,
                                    idx,
                                    desired_width * 0.5,
                                );
                                // save modification to toml_repo
                                if cmp_toml_repo(
                                    &self.toml_config.repos.as_ref().unwrap()[idx],
                                    &toml_repo,
                                ) {
                                    is_modified = true;
                                    self.toml_config.repos.as_mut().unwrap()[idx] = toml_repo;
                                }

                                // right panel - repository state
                                let repo_state = match idx < self.repo_states.len() {
                                    true => self.repo_states[idx].clone(),
                                    false => RepoState::default(),
                                };
                                self.repository_state_panel(ui, repo_state, desired_width * 0.5);
                            });
                        });
                        ui.separator();
                    }

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
    fn repository_remote_config_panel(
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
            let repository_display =
                format!("{} {}", hex_code::REPOSITORY, display_path(&rel_path));
            let job = match self.repo_states[idx].no_ignore {
                true => create_layout_job(repository_display, text_color::PURPLE),
                false => create_layout_job(repository_display, text_color::DARK_PURPLE),
            };

            ui.horizontal(|ui| {
                ui.set_row_height(18.0);
                // display name
                ui.label(job);

                let widget_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().max.x + 5.0, ui.min_rect().min.y),
                    egui::vec2(18.0, 12.0),
                );

                // open in file explorer
                let button_response =
                    ui.put(widget_rect, egui::Button::new(hex_code::LINK_EXTERNAL));
                if button_response.clicked() {
                    let full_path = format!("{}/{}", &self.project_path, &rel_path);
                    open_in_file_explorer(full_path);
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
                let pos = [ui.min_rect().min.x + 160.0, ui.min_rect().min.y - 40.0];
                ui.remote_ref_edit_button(
                    pos,
                    idx,
                    &mut branch_text,
                    &mut tag_text,
                    &mut commit_text,
                );
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

            // show remote url
            let url = format!(
                "{} {}",
                hex_code::URL,
                display_path(&toml_repo.remote.to_owned().unwrap())
            );
            let job = create_truncate_layout_job(url, text_color::LIGHT_GRAY);
            ui.label(job);
        });
    }

    /// part of app/content_view/repositories_list_panel
    fn repository_state_panel(
        &mut self,
        ui: &mut egui::Ui,
        repo_state: RepoState,
        desired_width: f32,
    ) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.set_width(desired_width);

            // show states
            if repo_state.err_msg.is_empty() {
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

                        ui.add_space(4.0);
                    }
                    // show Warning
                    StateType::Warning => {
                        let job = create_layout_job(
                            format!("{} Warning", hex_code::WARNING),
                            text_color::YELLOW,
                        );

                        ui.label(job);
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
                        // show normal
                        let job = create_layout_job(
                            format!("{} Normal", hex_code::NORMAL),
                            text_color::GREEN,
                        );
                        ui.label(job);
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
                        }
                        // Warning
                        else if repo_state.cmp_state == StateType::Warning {
                            let mut job = create_layout_job(
                                format!("{} {}", hex_code::COMMIT, &repo_state.cmp_commit),
                                text_color::YELLOW,
                            );
                            job.append(" ", 0.0, egui::TextFormat::default());
                            job.append(
                                &repo_state.cmp_changes,
                                0.0,
                                egui::TextFormat {
                                    color: text_color::RED,
                                    ..Default::default()
                                },
                            );
                            ui.label(job);
                        }
                        // Error
                        else {
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

fn get_repo_states_thread(
    toml_repos: Vec<TomlRepo>,
    project_path: String,
    default_branch: Option<String>,
    sender: Sender<(CommandType, (usize, RepoState))>,
) {
    std::thread::spawn(move || {
        let thread_pool = match rayon::ThreadPoolBuilder::new().build() {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let sender = Arc::new(Mutex::new(sender));
        thread_pool.install(|| {
            toml_repos.par_iter().for_each_with(&sender, |s, repo| {
                let toml_repos = toml_repos.clone();
                let idx = toml_repos
                    .into_iter()
                    .position(|r| r.local == repo.local)
                    .unwrap();

                let mut repo_state = RepoState::default();
                let input_path = Path::new(&project_path);
                let full_path = input_path.join(repo.to_owned().local.unwrap());

                let mut is_ok = true;
                if let Err(e) = is_repository(&full_path) {
                    repo_state.err_msg = e.to_string();
                    is_ok = false;
                }

                if is_ok {
                    // get current branch
                    match get_current_branch(&full_path) {
                        Ok(res) => {
                            repo_state.track_state = StateType::Normal;
                            repo_state.current_branch = res;
                        }
                        Err(e) => {
                            repo_state.err_msg = e.to_string();
                            is_ok = false;
                        }
                    }
                }

                if is_ok {
                    // get tracking branch
                    match get_tracking_branch(&full_path) {
                        Ok(res) => {
                            repo_state.tracking_branch = res;
                        }
                        Err(_) => {
                            repo_state.track_state = StateType::Warning;
                            repo_state.tracking_branch = "untracked".to_string();
                            is_ok = false;
                        }
                    }
                }

                if is_ok {
                    // get compare message
                    match cmp_local_remote(input_path, repo, &default_branch, true) {
                        Ok(cmp_msg) => {
                            let cmp_msg = String::from_utf8(
                                strip_ansi_escapes::strip(cmp_msg.unwrap()).unwrap(),
                            )
                            .unwrap();

                            if cmp_msg.clone().contains("not tracking")
                                || cmp_msg.contains("init commit")
                                || cmp_msg.contains("unknown revision")
                            {
                                repo_state.cmp_state = StateType::Error;
                                repo_state.cmp_obj = cmp_msg.clone();
                            } else if cmp_msg.clone().contains("already update to date.") {
                                repo_state.cmp_state = StateType::Normal;
                                let (prefix, log) = cmp_msg.split_once('.').unwrap();
                                repo_state.cmp_obj = log.trim().to_string();
                                if repo_state.cmp_obj.is_empty() {
                                    repo_state.cmp_obj = prefix.to_string();
                                }
                            } else {
                                repo_state.cmp_state = StateType::Warning;
                                for part in cmp_msg.clone().split(",") {
                                    if part.contains("commits") {
                                        repo_state.cmp_commit = part.trim().to_string()
                                    } else if part.contains("changes") {
                                        repo_state.cmp_changes = part.trim().to_string()
                                    } else {
                                        repo_state.cmp_obj = part.trim().to_string();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            repo_state.err_msg = e.to_string();
                        }
                    }
                }

                s.lock()
                    .unwrap()
                    .send((CommandType::None, (idx, repo_state)))
                    .unwrap();
            })
        });
    });
}

fn execute_cmd_with_send(
    cmd: &str,
    project: &str,
    option: &str,
    command_type: CommandType,
    send: Sender<(CommandType, (usize, RepoState))>,
) {
    let command_str = format!("{} {} \"{}\" {}", MGIT_DIR, cmd, project, option);
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .raw_arg("/C")
                .raw_arg(&command_str)
                .creation_flags(defines::console_option::DETACHED_PROCESS)
                .output()
                .expect("command failed to start");
        }

        #[cfg(target_os = "macos")]
        {
            let cur_path = std::env::current_exe().unwrap();
            let cur_path = cur_path.parent().unwrap().to_str().unwrap();
            let command_str = format!("./{}", &command_str);
            std::process::Command::new("sh")
                .current_dir(cur_path)
                .arg("-c")
                .arg(&command_str)
                .output()
                .expect("command failed to start");
        }

        send.send((command_type, (usize::MAX, RepoState::default())))
            .unwrap();
    });
}

impl<'t> UiExt<'t> for egui::Ui {
    fn remote_ref_edit_button(
        &mut self,
        current_pos: impl Into<egui::Pos2>,
        idx: usize,
        branch_text: &'t mut dyn egui::TextBuffer,
        tag_text: &'t mut dyn egui::TextBuffer,
        commit_text: &'t mut dyn egui::TextBuffer,
    ) -> egui::Response {
        let source = format!("remote_ref_editing_context_{}", idx);
        let popup_id = egui::Id::new(self.skip_ahead_auto_ids(0)).with(source);
        let open = self.memory().is_popup_open(popup_id);

        let widget_rect = egui::Rect::from_min_size(
            egui::pos2(self.min_rect().max.x + 5.0, self.min_rect().min.y),
            egui::vec2(18.0, 18.0),
        );

        let toggle_response = self.put(
            widget_rect,
            egui::SelectableLabel::new(open, hex_code::EDIT),
        );

        if toggle_response.clicked() {
            self.memory().toggle_popup(popup_id);
        }

        if self.memory().is_popup_open(popup_id) {
            let area_response = egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .current_pos(current_pos)
                .show(self.ctx(), |ui| {
                    egui::Frame::popup(self.style()).show(ui, |ui| {
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
                                ui.add_sized(label_size, egui::TextEdit::singleline(branch_text));

                                ui.end_row();

                                // tag
                                ui.label(format!("  {} tag", hex_code::TAG));

                                ui.add_sized(label_size, egui::TextEdit::singleline(tag_text));
                                ui.end_row();

                                // commit
                                ui.label(format!("  {} commmit", hex_code::COMMIT));
                                ui.add_sized(label_size, egui::TextEdit::singleline(commit_text));
                                ui.end_row();
                            });

                        ui.add_space(5.0);
                    });
                })
                .response;

            if !toggle_response.clicked()
                && (self.input().key_pressed(egui::Key::Escape)
                    || area_response.clicked_elsewhere()
                    || (self.input().scroll_delta.y.abs() > 0.0 && !area_response.hovered()))
            {
                self.memory().close_popup();
            }
        }
        toggle_response
    }
}
