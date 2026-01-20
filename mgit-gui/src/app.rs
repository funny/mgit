use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Instant;

use eframe::egui;
use egui::Visuals;
use tracing::{debug, info, warn};

use crate::app::context::{AppContext, PendingConfigSave, RepoState};
use crate::app::events::{Action, BackendEvent, CommandType, Event, InputEvent};
use crate::ui::windows::{ErrorWindow, OptionsWindow, WindowManager};

pub mod context;
pub mod events;
pub mod repo_manager;
pub mod session_manager;

use crate::ui::style::{configure_text_styles, setup_custom_fonts};
use crate::utils::system::check_git_valid;

use crate::app::repo_manager::RepoManager;
use crate::app::session_manager::SessionManager;

use mgit::utils::path::PathExtension;

pub struct GuiApp {
    pub(crate) context: egui::Context,
    pub(crate) app_context: AppContext, // Renamed to app_context to avoid confusion with egui::Context

    pub(crate) event_rx: Receiver<Event>,
    pub(crate) windows: WindowManager,
    pub(crate) queued_events: VecDeque<Event>,
    pub(crate) first_frame: bool,
}

impl Default for GuiApp {
    fn default() -> Self {
        let (event_tx, event_rx) = channel();
        let progress = Arc::new(AtomicUsize::new(0));

        let app_context = AppContext {
            repo_manager: RepoManager::new(event_tx.clone(), progress),
            session_manager: SessionManager::new(),
            event_tx,
            run_id_seq: AtomicU64::new(0),
        };

        Self {
            context: egui::Context::default(),
            app_context,
            event_rx,
            windows: WindowManager::default(),
            queued_events: VecDeque::new(),
            first_frame: false,
        }
    }
}

impl GuiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        setup_custom_fonts(&cc.egui_ctx);
        configure_text_styles(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(Visuals::dark());

        let mut app = GuiApp {
            context: cc.egui_ctx.clone(),
            first_frame: true,
            ..GuiApp::default()
        };

        let (is_dependencies_valid, err_msg) = match check_git_valid() {
            Ok(_) => (true, String::new()),
            Err(msg) => (false, msg),
        };

        if !is_dependencies_valid {
            app.windows.error_open = true;
            app.windows.error_exit_app = true;
            app.windows.error = ErrorWindow::new(err_msg);
            return app;
        }

        app.app_context.session_manager.load_setting();
        app.app_context.repo_manager.exec_ops(
            app.app_context.next_run_id(),
            CommandType::Refresh,
            &mut app.app_context.session_manager,
        );

        app
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        if self.first_frame {
            ctx.set_visuals(Visuals::dark());
            self.first_frame = false;
        }

        self.top_view(ctx);
        self.content_view(ctx);
        self.handle_windows(ctx, eframe);

        self.drain_event_channel();
        self.app_context.repo_manager.flush_config_save_if_due();
        self.process_events(ctx);
    }
}

impl GuiApp {
    pub(crate) fn enqueue_event(&mut self, event: Event) {
        self.queued_events.push_back(event);
        debug!(queued_len = self.queued_events.len(), "event_enqueued");
    }

    pub(crate) fn drain_event_channel(&mut self) {
        let mut drained = 0usize;
        while let Ok(event) = self.event_rx.try_recv() {
            self.enqueue_event(event);
            drained += 1;
        }
        if drained > 0 {
            debug!(
                drained,
                queued_len = self.queued_events.len(),
                "event_channel_drained"
            );
        }
    }

    pub(crate) fn process_events(&mut self, ctx: &egui::Context) {
        while let Some(event) = self.queued_events.pop_front() {
            self.handle_event(event, ctx);
        }
    }

    pub(crate) fn handle_event(&mut self, event: Event, ctx: &egui::Context) {
        let started_at = Instant::now();
        info!("Processing Event: {:?}", event);
        match event {
            Event::Input(input) => self.handle_input(input),
            Event::Action(action) => self.handle_action(action, ctx),
            Event::Backend(event) => self.apply_backend_event(event, ctx),
        }
        debug!(
            duration_ms = started_at.elapsed().as_millis(),
            "event_processed"
        );
    }

    fn handle_input(&mut self, input: InputEvent) {
        match input {
            InputEvent::ProjectPathChanged(path) => {
                info!(path = path.as_str(), "input_project_path_changed");
                self.app_context.session_manager.project_path = Path::new(&path).norm_path();
                self.app_context.session_manager.push_recent_project();
                self.app_context.session_manager.load_project_settings();

                self.windows.options = OptionsWindow::default();
                self.windows.options.load_option_from_settings(
                    &self.app_context.session_manager.user_settings,
                    &self.app_context.session_manager.get_snapshot_ignore(),
                );

                self.app_context.session_manager.config_file =
                    match &self.app_context.session_manager.get_recent_configs() {
                        Some(recent_configs) if !recent_configs.is_empty() => {
                            recent_configs[0].clone()
                        }
                        _ => format!(
                            "{}/.gitrepos",
                            &self.app_context.session_manager.project_path
                        ),
                    };
                self.enqueue_event(Event::Input(InputEvent::ConfigFileChanged(
                    self.app_context.session_manager.config_file.clone(),
                )));
            }
            InputEvent::ConfigFileChanged(path) => {
                info!(path = path.as_str(), "input_config_file_changed");
                self.app_context.session_manager.config_file = Path::new(&path).norm_path();
                if Path::new(&self.app_context.session_manager.config_file).is_file() {
                    self.app_context.session_manager.push_recent_config();
                }
                self.enqueue_event(Event::Action(Action::Refresh));
            }
        }
    }

    fn handle_action(&mut self, action: Action, _ctx: &egui::Context) {
        match action {
            Action::RunOps(command) => {
                let run_id = self.app_context.next_run_id();
                info!(run_id, command = ?command, "action_run_ops");
                self.app_context.repo_manager.exec_ops_command(
                    run_id,
                    command,
                    &mut self.app_context.session_manager,
                    &mut self.windows,
                )
            }
            Action::RunOpsBatch(commands) => {
                let run_id = self.app_context.next_run_id();
                info!(
                    run_id,
                    command_count = commands.len(),
                    "action_run_ops_batch"
                );
                commands.into_iter().for_each(|command| {
                    self.app_context.repo_manager.exec_ops_command(
                        run_id,
                        command,
                        &mut self.app_context.session_manager,
                        &mut self.windows,
                    )
                })
            }
            Action::Refresh => {
                let run_id = self.app_context.next_run_id();
                info!(run_id, "action_refresh");
                self.app_context.repo_manager.exec_ops(
                    run_id,
                    CommandType::Refresh,
                    &mut self.app_context.session_manager,
                )
            }
            Action::RetryConfigSave => {
                info!("action_retry_config_save");
                self.app_context.repo_manager.retry_last_config_save()
            }
            Action::SaveOptions => {
                info!("action_save_options");
                self.app_context
                    .session_manager
                    .user_settings
                    .save_options(&self.windows.options.to_gui_options())
            }
            Action::SaveSnapshotIgnore => {
                info!("action_save_snapshot_ignore");
                let ignore = self.windows.options.snapshot_ignore.clone();
                self.app_context
                    .session_manager
                    .save_snapshot_ignore(ignore);
            }
            Action::SaveNewBranchOption => {
                info!("action_save_new_branch_option");
                self.app_context
                    .session_manager
                    .save_new_branch_option_from_window(&self.windows.new_branch);
            }
            Action::SaveNewTagOption => {
                info!("action_save_new_tag_option");
                self.app_context
                    .session_manager
                    .save_new_tag_option_from_window(&self.windows.new_tag);
            }
            Action::ExitApp => {
                warn!("action_exit_app");
                std::process::exit(0x0100)
            }
        }
    }

    fn apply_backend_event(&mut self, event: BackendEvent, _ctx: &egui::Context) {
        match event {
            BackendEvent::RepoStateUpdated {
                run_id,
                id,
                repo_state,
            } => {
                debug!(run_id, repo_id = id, "repo_state_updated");
                if let Some(existing) = self.app_context.repo_manager.repo_states.get(id).cloned() {
                    self.app_context.repo_manager.repo_states[id] = RepoState {
                        no_ignore: existing.no_ignore,
                        disable_by_label: existing.disable_by_label,
                        ..repo_state
                    };
                }
            }
            BackendEvent::CommandFinished { run_id, command } => {
                let kind = command.kind();
                if kind != CommandType::None {
                    if kind != CommandType::Refresh {
                        self.app_context
                            .repo_manager
                            .load_config(Path::new(&self.app_context.session_manager.config_file));
                    }
                    self.app_context
                        .repo_manager
                        .reset_repo_state(crate::app::repo_manager::StateType::Updating);
                    self.app_context.repo_manager.get_repo_states(
                        run_id,
                        self.context.clone(),
                        self.app_context.session_manager.project_path.clone(),
                    );
                }
            }
            BackendEvent::RemoteBranchesLoaded {
                run_id,
                repo_rel_path,
                branches,
            } => {
                info!(
                    run_id,
                    repo_rel_path = repo_rel_path.as_str(),
                    branch_count = branches.len(),
                    "remote_branches_loaded"
                );
                let entry = self
                    .app_context
                    .repo_manager
                    .remote_branches_cache
                    .entry(repo_rel_path)
                    .or_default();
                entry.loading = false;
                entry.error = None;
                entry.branches = branches;
            }
            BackendEvent::RemoteBranchesFailed {
                run_id,
                repo_rel_path,
                error,
            } => {
                warn!(
                    run_id,
                    repo_rel_path = repo_rel_path.as_str(),
                    error = error.as_str(),
                    "remote_branches_failed"
                );
                let entry = self
                    .app_context
                    .repo_manager
                    .remote_branches_cache
                    .entry(repo_rel_path)
                    .or_default();
                entry.loading = false;
                entry.error = Some(error);
                entry.branches.clear();
            }
            BackendEvent::ConfigSaved { run_id, path } => {
                info!(run_id, path = path.as_str(), "config_saved");
                self.app_context.repo_manager.config_save_inflight = false;
                self.app_context.repo_manager.last_failed_config_save = None;
                self.app_context.repo_manager.remote_branches_cache.clear();
            }
            BackendEvent::ConfigSaveFailed {
                run_id,
                path,
                content,
                error,
            } => {
                warn!(
                    run_id,
                    path = path.as_str(),
                    content_len = content.len(),
                    error = error.as_str(),
                    "config_save_failed"
                );
                self.app_context.repo_manager.config_save_inflight = false;
                self.app_context.repo_manager.last_failed_config_save = Some(PendingConfigSave {
                    run_id,
                    path: path.clone(),
                    content,
                    scheduled_at: Instant::now(),
                });
                self.windows.error_exit_app = false;
                self.windows.error_open = true;
                self.windows.error = ErrorWindow::new_retryable(format!(
                    "Failed to save config: {}\n{}",
                    path, error
                ));
            }
            BackendEvent::ConfigLoadFailed { run_id, error } => {
                warn!(run_id, error = error.as_str(), "config_load_failed");
                self.windows.error_exit_app = false;
                self.windows.error_open = true;
                self.windows.error = ErrorWindow::new(format!("Failed to load config:\n{}", error));
            }
        }
        self.context.request_repaint();
    }
}
