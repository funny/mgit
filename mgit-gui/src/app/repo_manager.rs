use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::events::CommandType;
use crate::app::events::{BackendEvent, Event, OpsCommand};
use crate::app::session_manager::SessionManager;
use crate::configs::SyncType;
use crate::ui::windows::WindowManager;
use crate::utils::fs;
use crate::utils::progress::OpsMessageCollector;
use mgit::config::{cmp_local_remote, MgitConfig, RepoConfig};
use mgit::git;
use mgit::ops;
use mgit::ops::{
    CleanOptions, FetchOptions, InitOptions, SnapshotOptions, SnapshotType, SyncOptions,
    TrackOptions,
};
use mgit::utils::path::PathExtension;
use sha256::digest;
use tracing::{debug, error, info};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StateType {
    Disconnected,
    Updating,
    Normal,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct RepoState {
    pub current_branch: String,
    pub tracking_branch: String,
    pub track_state: StateType,
    pub tags: Vec<String>,
    pub cmp_obj: String,
    pub cmp_commit: String,
    pub cmp_changes: String,
    pub cmp_state: StateType,
    pub err_msg: String,
    pub no_ignore: bool,
    pub disable_by_label: bool,
}

impl Default for RepoState {
    fn default() -> Self {
        Self {
            current_branch: String::new(),
            tracking_branch: String::new(),
            track_state: StateType::Disconnected,
            tags: Vec::new(),
            cmp_obj: String::new(),
            cmp_commit: String::new(),
            cmp_changes: String::new(),
            cmp_state: StateType::Disconnected,
            err_msg: String::new(),
            no_ignore: true,
            disable_by_label: false,
        }
    }
}

impl RepoState {
    pub fn is_disable(&self) -> bool {
        !self.no_ignore || self.disable_by_label
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemoteBranchesCacheEntry {
    pub loading: bool,
    pub branches: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PendingConfigSave {
    pub run_id: u64,
    pub path: String,
    pub content: String,
    pub scheduled_at: Instant,
}

pub struct RepoManager {
    pub mgit_config: MgitConfig,
    pub repo_states: Vec<RepoState>,
    pub remote_branches_cache: HashMap<String, RemoteBranchesCacheEntry>,
    pub remote_ref_edit_idx: i32,

    pub pending_config_save: Option<PendingConfigSave>,
    pub last_failed_config_save: Option<PendingConfigSave>,
    pub config_save_inflight: bool,

    pub event_tx: Sender<Event>,
    pub progress: Arc<AtomicUsize>,
    pub ops_message_collector: OpsMessageCollector,
}

impl RepoManager {
    pub fn new(event_tx: Sender<Event>, progress: Arc<AtomicUsize>) -> Self {
        Self {
            mgit_config: MgitConfig::default(),
            repo_states: Vec::new(),
            remote_branches_cache: HashMap::new(),
            remote_ref_edit_idx: -1,
            pending_config_save: None,
            last_failed_config_save: None,
            config_save_inflight: false,
            event_tx: event_tx.clone(),
            progress: progress.clone(),
            ops_message_collector: OpsMessageCollector::new(event_tx, progress),
        }
    }

    pub fn load_config(&mut self, config_file: &Path) {
        self.remote_ref_edit_idx = -1;
        if config_file.is_file() {
            match MgitConfig::load(config_file) {
                Some(mgit_config) => {
                    self.mgit_config = mgit_config;
                }
                None => {
                    let _ = self
                        .event_tx
                        .send(Event::Backend(BackendEvent::ConfigLoadFailed {
                            run_id: 0,
                            error: format!("Failed to load config: {}", config_file.display_path()),
                        }));
                }
            };
        }
        self.remote_branches_cache.clear();
        self.sync_repo_states_and_collector();
    }

    pub fn clear_mgit_config(&mut self) {
        self.mgit_config = MgitConfig::default();
        self.repo_states.clear();
        let empty = Vec::new();
        self.ops_message_collector.update(&empty);
    }

    pub fn recompute_repo_filters(
        &mut self,
        ignores: Option<&Vec<String>>,
        labels: Option<&Vec<String>>,
    ) {
        let Some(repo_configs) = &self.mgit_config.repos else {
            return;
        };

        let ignores = ignores.cloned().unwrap_or_default();
        for (repo_config, state) in repo_configs.iter().zip(&mut self.repo_states) {
            let rel_path = repo_config
                .local
                .as_ref()
                .map_or(String::from("invalid"), |p| p.clone());

            let do_ignore = ignores.contains(&rel_path.display_path());
            state.no_ignore = !do_ignore;
            state.disable_by_label = match &labels {
                Some(labels) => !mgit::utils::label::check(repo_config, labels),
                None => false,
            };
        }
    }

    fn sync_repo_states_and_collector(&mut self) {
        self.repo_states.clear();
        let Some(repo_configs) = &self.mgit_config.repos else {
            let empty = Vec::new();
            self.ops_message_collector.update(&empty);
            return;
        };

        self.repo_states
            .resize_with(repo_configs.len(), RepoState::default);
        self.ops_message_collector.update(repo_configs);
    }

    pub fn schedule_config_save(&mut self, run_id: u64, config_file: String, content: String) {
        let replaced = self
            .pending_config_save
            .as_ref()
            .map(|p| (p.run_id, p.path.clone()));
        let content_len = content.len();
        let content_hash = digest(content.as_bytes());
        self.pending_config_save = Some(PendingConfigSave {
            run_id,
            path: config_file,
            content,
            scheduled_at: Instant::now(),
        });
        if let Some((replaced_run_id, replaced_path)) = replaced {
            debug!(
                run_id,
                path = self
                    .pending_config_save
                    .as_ref()
                    .map(|p| p.path.as_str())
                    .unwrap_or(""),
                content_len,
                content_hash = content_hash.as_str(),
                replaced_run_id,
                replaced_path = replaced_path.as_str(),
                "config_save_scheduled_replaced"
            );
        } else {
            debug!(
                run_id,
                path = self
                    .pending_config_save
                    .as_ref()
                    .map(|p| p.path.as_str())
                    .unwrap_or(""),
                content_len,
                content_hash = content_hash.as_str(),
                "config_save_scheduled"
            );
        }
    }

    pub fn retry_last_config_save(&mut self) {
        if self.config_save_inflight {
            return;
        }
        let Some(mut pending) = self.last_failed_config_save.clone() else {
            return;
        };
        info!(
            run_id = pending.run_id,
            path = pending.path.as_str(),
            content_len = pending.content.len(),
            "config_save_retry_scheduled"
        );
        pending.scheduled_at = Instant::now() - Duration::from_millis(1000);
        self.pending_config_save = Some(pending);
    }

    pub fn flush_config_save_if_due(&mut self) {
        if self.config_save_inflight {
            return;
        }

        let due = match self.pending_config_save.as_ref() {
            Some(pending) => pending.scheduled_at.elapsed() >= Duration::from_millis(500),
            None => false,
        };
        if !due {
            return;
        }

        let Some(pending) = self.pending_config_save.take() else {
            return;
        };

        self.config_save_inflight = true;
        let sender = self.event_tx.clone();
        std::thread::spawn(move || {
            let started_at = Instant::now();
            let run_id = pending.run_id;
            let path = pending.path;
            let content = pending.content;
            let content_len = content.len();
            let content_hash = digest(content.as_bytes());
            info!(
                run_id,
                path = path.as_str(),
                content_len,
                content_hash = content_hash.as_str(),
                "config_save_flush_start"
            );
            let result = fs::atomic_write_file(Path::new(&path), content.as_bytes());
            match result {
                Ok(_) => {
                    info!(
                        run_id,
                        path = path.as_str(),
                        duration_ms = started_at.elapsed().as_millis(),
                        "config_save_flush_ok"
                    );
                    let _ = sender.send(Event::Backend(BackendEvent::ConfigSaved { run_id, path }));
                }
                Err(e) => {
                    error!(
                        run_id,
                        path = path.as_str(),
                        duration_ms = started_at.elapsed().as_millis(),
                        error = %e,
                        "config_save_flush_failed"
                    );
                    let _ = sender.send(Event::Backend(BackendEvent::ConfigSaveFailed {
                        run_id,
                        path,
                        content,
                        error: e.to_string(),
                    }));
                }
            }
        });
    }

    pub(crate) fn exec_ops_command(
        &mut self,
        run_id: u64,
        command: OpsCommand,
        session: &mut SessionManager,
        _windows: &mut WindowManager,
    ) {
        info!(run_id, command = ?command, "ops_command_requested");
        match command {
            OpsCommand::Simple(kind) => self.exec_ops(run_id, kind, session),
            OpsCommand::Snapshot { config_file } => {
                session.config_file = config_file.norm_path();
                self.exec_ops(run_id, CommandType::Snapshot, session);
            }
            OpsCommand::CreateBranch(options) => self.exec_new_branch(run_id, options, session),
            OpsCommand::CreateTag(options) => self.exec_new_tag(run_id, options),
        }
    }

    fn exec_new_branch(
        &mut self,
        run_id: u64,
        options: ops::NewBranchOptions,
        session: &mut SessionManager,
    ) {
        if let Some(path) = options.new_config_path.as_ref() {
            session.config_file = path.norm_path();
            session.push_recent_config();
        }

        let send = self.event_tx.clone();
        self.clear_status();

        std::thread::spawn(move || {
            let started_at = Instant::now();
            info!(run_id, "ops_new_branch_started");
            let result = crate::utils::runtime::block_on(ops::new_remote_branch(options));
            match result {
                Ok(msg) => info!(run_id, message = msg.to_plain_text(), "ops_new_branch_ok"),
                Err(e) => error!(run_id, error = %e, "ops_new_branch_failed"),
            }
            info!(
                run_id,
                duration_ms = started_at.elapsed().as_millis(),
                "ops_new_branch_finished"
            );

            send.send(Event::Backend(BackendEvent::CommandFinished {
                run_id,
                command: CommandType::NewBranch.into(),
            }))
            .unwrap();
        });
    }

    fn exec_new_tag(&mut self, run_id: u64, options: ops::NewTagOptions) {
        let send = self.event_tx.clone();
        self.clear_status();

        std::thread::spawn(move || {
            let started_at = Instant::now();
            info!(run_id, "ops_new_tag_started");
            let result = crate::utils::runtime::block_on(ops::new_tag(options));
            match result {
                Ok(msg) => info!(run_id, message = msg.to_plain_text(), "ops_new_tag_ok"),
                Err(e) => error!(run_id, error = %e, "ops_new_tag_failed"),
            }
            info!(
                run_id,
                duration_ms = started_at.elapsed().as_millis(),
                "ops_new_tag_finished"
            );

            send.send(Event::Backend(BackendEvent::CommandFinished {
                run_id,
                command: CommandType::NewTag.into(),
            }))
            .unwrap();
        });
    }

    pub(crate) fn exec_ops(
        &mut self,
        run_id: u64,
        command_type: CommandType,
        session: &mut SessionManager,
    ) {
        // to show in ui
        if session.config_file.is_empty() {
            session.config_file = format!("{}/.gitrepos", &session.project_path);
        }

        self.progress = Arc::new(AtomicUsize::new(0));
        match command_type {
            CommandType::Init => {
                session.config_file = format!("{}/.gitrepos", &session.project_path);

                let path = Some(session.project_path.clone());
                let force = session.user_settings.init_force;

                let options = InitOptions::new(path.as_deref(), force);
                let send = self.event_tx.clone();
                self.clear_status();
                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    info!(run_id, "ops_init_started");
                    let result = crate::utils::runtime::block_on(ops::init_repo(options));
                    match result {
                        Ok(msg) => info!(run_id, message = msg.to_plain_text(), "ops_init_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_init_failed"),
                    }
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_init_finished"
                    );
                    send.send(Event::Backend(BackendEvent::CommandFinished {
                        run_id,
                        command: command_type.into(),
                    }))
                    .unwrap();
                });
            }

            CommandType::Snapshot => {
                let path = Some(session.project_path.clone());
                let config_path = Some(session.config_file.clone());
                let snapshot_type = session.user_settings.snapshot_branch.and_then(|b| match b {
                    true => Some(SnapshotType::Branch),
                    false => None,
                });
                let force = session.user_settings.snapshot_force;
                let ignore: Option<Vec<String>> = session
                    .get_snapshot_ignore()
                    .map(|content| content.split('\n').map(|s| s.to_string()).collect());

                let options = SnapshotOptions::new(
                    path.as_deref(),
                    config_path.as_deref(),
                    force,
                    snapshot_type,
                    ignore,
                );
                let send = self.event_tx.clone();

                session.push_recent_config();
                self.clear_status();

                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    info!(run_id, "ops_snapshot_started");
                    let result = crate::utils::runtime::block_on(ops::snapshot_repo(options));
                    match result {
                        Ok(msg) => info!(run_id, message = msg.to_plain_text(), "ops_snapshot_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_snapshot_failed"),
                    }
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_snapshot_finished"
                    );
                    send.send(Event::Backend(BackendEvent::CommandFinished {
                        run_id,
                        command: command_type.into(),
                    }))
                    .unwrap();
                });
            }

            CommandType::Fetch => {
                let path = Some(session.project_path.clone());
                let config_path = Some(session.config_file.clone());
                let thread = session.user_settings.sync_thread.map(|t| t as usize);
                let depth = session.user_settings.sync_depth.map(|d| d as usize);
                let ignore: Option<Vec<String>> = session.get_ignores();
                let labels = session.get_labels();
                let silent = Some(true);

                info!(
                    run_id,
                    command = ?command_type,
                    project_path = session.project_path.as_str(),
                    config_file = session.config_file.as_str(),
                    thread,
                    depth,
                    ignore_count = ignore.as_ref().map(|v| v.len()).unwrap_or(0),
                    labels_count = labels.as_ref().map(|v| v.len()).unwrap_or(0),
                    "ops_start"
                );
                let options = FetchOptions::new(
                    path.as_deref(),
                    config_path.as_deref(),
                    thread,
                    silent,
                    depth,
                    ignore,
                    labels,
                );

                self.reset_repo_state(StateType::Updating);
                let progress = self.progress(run_id, command_type, &session.project_path);
                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    let result = crate::utils::runtime::block_on(ops::fetch_repos(
                        options,
                        progress.clone(),
                    ));
                    match result {
                        Ok(msg) => debug!(run_id, message = msg.to_plain_text(), "ops_fetch_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_fetch_failed"),
                    }
                    progress.send_command_finished_once();
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_finished"
                    );
                });
            }

            CommandType::Sync | CommandType::SyncHard => {
                let path = Some(session.project_path.clone());
                let config_path = Some(session.config_file.clone());
                // check if command_type is CommandType::SyncHard
                let sync_type = match command_type == CommandType::SyncHard {
                    true => SyncType::Hard,
                    false => session.user_settings.sync_type.unwrap_or(SyncType::Stash),
                };
                // option none or --stash or --hard
                let (hard, stash) = match sync_type {
                    SyncType::Normal => (Some(false), Some(false)),
                    SyncType::Stash => (Some(false), Some(true)),
                    SyncType::Hard => (Some(true), Some(false)),
                };
                // option --no-checkout
                let no_checkout = session.user_settings.sync_no_checkout;
                // option --no-track
                let no_track = session.user_settings.sync_no_track;
                // option --thread <num>
                let thread_count = session.user_settings.sync_thread.map(|t| t as usize);
                // option --depth <num>
                let depth = session.user_settings.sync_depth.map(|d| d as usize);
                // option --ignore
                let ignore: Option<Vec<String>> = session.get_ignores();
                // option --labels
                let labels = session.get_labels();
                // option --silent
                let silent = Some(true);

                info!(
                    run_id,
                    command = ?command_type,
                    project_path = session.project_path.as_str(),
                    config_file = session.config_file.as_str(),
                    sync_type = ?sync_type,
                    thread_count,
                    depth,
                    ignore_count = ignore.as_ref().map(|v| v.len()).unwrap_or(0),
                    labels_count = labels.as_ref().map(|v| v.len()).unwrap_or(0),
                    hard,
                    stash,
                    no_track,
                    no_checkout,
                    "ops_start"
                );
                let options = SyncOptions::new(
                    path.as_deref(),
                    config_path.as_deref(),
                    thread_count,
                    silent,
                    depth,
                    ignore,
                    labels,
                    hard,
                    stash,
                    no_track,
                    no_checkout,
                );

                self.reset_repo_state(StateType::Updating);
                let progress = self.progress(run_id, command_type, &session.project_path);

                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    let result =
                        crate::utils::runtime::block_on(ops::sync_repo(options, progress.clone()));
                    match result {
                        Ok(msg) => debug!(run_id, message = msg.to_plain_text(), "ops_sync_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_sync_failed"),
                    }
                    progress.send_command_finished_once();
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_finished"
                    );
                });
            }

            CommandType::Track => {
                let path = Some(session.project_path.clone());
                let config_path = Some(session.config_file.clone());
                let ignore: Option<Vec<String>> = session.get_ignores();

                let options = TrackOptions::new(path.as_deref(), config_path.as_deref(), ignore);

                self.reset_repo_state(StateType::Updating);
                let progress = self.progress(run_id, command_type, &session.project_path);

                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    let result =
                        crate::utils::runtime::block_on(ops::track(options, progress.clone()));
                    match result {
                        Ok(msg) => debug!(run_id, message = msg.to_plain_text(), "ops_track_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_track_failed"),
                    }
                    progress.send_command_finished_once();
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_finished"
                    );
                });
            }

            CommandType::Clean => {
                let path = Some(session.project_path.clone());
                let config_path = Some(session.config_file.clone());
                // option --labels
                let labels = session.get_labels();

                let options = CleanOptions::new(path.as_deref(), config_path.as_deref(), labels);
                let send = self.event_tx.clone();

                self.reset_repo_state(StateType::Updating);

                std::thread::spawn(move || {
                    let started_at = Instant::now();
                    info!(run_id, "ops_clean_started");
                    let result = crate::utils::runtime::block_on(ops::clean_repo(options));
                    match result {
                        Ok(msg) => info!(run_id, message = msg.to_plain_text(), "ops_clean_ok"),
                        Err(e) => error!(run_id, error = %e, "ops_clean_failed"),
                    }
                    info!(
                        run_id,
                        duration_ms = started_at.elapsed().as_millis(),
                        "ops_clean_finished"
                    );
                    send.send(Event::Backend(BackendEvent::CommandFinished {
                        run_id,
                        command: command_type.into(),
                    }))
                    .unwrap();
                });
            }

            CommandType::Refresh => {
                self.progress.store(0, Ordering::Relaxed);
                self.clear_status();
                self.load_config(Path::new(&session.config_file));
                self.recompute_repo_filters(
                    session.get_ignores().as_ref(),
                    session.get_labels().as_ref(),
                );
                self.reset_repo_state(StateType::Updating);
                let _ = self
                    .event_tx
                    .send(Event::Backend(BackendEvent::CommandFinished {
                        run_id,
                        command: command_type.into(),
                    }));
            }

            CommandType::NewBranch => {
                // Should use exec_new_branch
            }

            CommandType::NewTag => {
                // Should use exec_new_tag
            }

            CommandType::None => {}
        }
    }

    fn progress(
        &mut self,
        run_id: u64,
        command_type: CommandType,
        project_path: &str,
    ) -> OpsMessageCollector {
        self.ops_message_collector.progress = self.progress.clone();
        self.ops_message_collector.command_type = command_type;
        self.ops_message_collector.run_id = run_id;
        self.ops_message_collector.project_path = project_path.to_string();
        self.ops_message_collector.default_branch = self.mgit_config.default_remote.clone();
        self.ops_message_collector.clone()
    }

    fn clear_status(&mut self) {
        self.clear_repo_state();
        self.clear_mgit_config();
    }

    pub(crate) fn clear_repo_state(&mut self) {
        self.repo_states = Vec::new();
    }

    pub(crate) fn reset_repo_state(&mut self, state_type: StateType) {
        for repo_state in &mut self.repo_states {
            if repo_state.is_disable() {
                continue;
            }
            *repo_state = RepoState {
                track_state: state_type,
                cmp_state: state_type,
                no_ignore: repo_state.no_ignore,
                disable_by_label: repo_state.disable_by_label,
                ..RepoState::default()
            };
        }
    }

    pub(crate) fn get_repo_states(
        &mut self,
        run_id: u64,
        ctx: eframe::egui::Context,
        project_path: String,
    ) {
        // get repository state
        if let Some(repos) = self.mgit_config.repos.as_ref() {
            let default_branch = self.mgit_config.default_branch.clone();
            get_repo_states_parallel(
                run_id,
                repos.clone(),
                project_path,
                default_branch,
                self.event_tx.clone(),
                ctx,
            )
        }
    }
}

fn get_repo_states_parallel(
    run_id: u64,
    repo_configs: Vec<RepoConfig>,
    project_path: String,
    default_branch: Option<String>,
    sender: Sender<Event>,
    ctx: eframe::egui::Context,
) {
    std::thread::spawn(move || {
        let project_path = Arc::new(project_path);
        let default_branch = Arc::new(default_branch);
        let repo_configs = Arc::new(repo_configs);

        let worker_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let mut handles = Vec::with_capacity(worker_count);
        for worker_id in 0..worker_count {
            let sender = sender.clone();
            let project_path = project_path.clone();
            let default_branch = default_branch.clone();
            let repo_configs = repo_configs.clone();
            handles.push(std::thread::spawn(move || {
                let rt = crate::utils::runtime::runtime();
                let mut id = worker_id;
                while id < repo_configs.len() {
                    let repo = &repo_configs[id];
                    let repo_state = get_repo_state_with_runtime(
                        repo,
                        project_path.as_ref(),
                        default_branch.as_ref(),
                        rt,
                    );
                    let _ = sender.send(Event::Backend(BackendEvent::RepoStateUpdated {
                        run_id,
                        id,
                        repo_state,
                    }));
                    id += worker_count;
                }
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        // NOTE: 保证所有仓库忽略后正常渲染
        ctx.request_repaint();
    });
}

pub(crate) fn get_repo_state(
    repo: &RepoConfig,
    project_path: &String,
    default_branch: &Option<String>,
) -> RepoState {
    let rt = crate::utils::runtime::runtime();
    get_repo_state_with_runtime(repo, project_path, default_branch, rt)
}

fn get_repo_state_with_runtime(
    repo: &RepoConfig,
    project_path: &String,
    default_branch: &Option<String>,
    rt: &tokio::runtime::Runtime,
) -> RepoState {
    let mut repo_state = RepoState::default();
    let input_path = Path::new(&project_path);
    let Some(local) = repo.local.as_ref() else {
        repo_state.err_msg = "invalid repo: local is missing".to_string();
        repo_state.track_state = StateType::Error;
        repo_state.cmp_state = StateType::Error;
        return repo_state;
    };
    let full_path = input_path.join(local);

    let mut is_ok = true;
    if let Err(e) = rt.block_on(git::is_repository(&full_path)) {
        repo_state.err_msg = e.to_string();
        is_ok = false;
    }

    if let Some(remote_url) = repo.remote.as_ref() {
        if let Err(e) = rt.block_on(git::find_remote_name_by_url(&full_path, remote_url)) {
            repo_state.err_msg = e.to_string();
            is_ok = false;
        }
    }

    if is_ok {
        // get tags
        match rt.block_on(git::get_head_tags(&full_path)) {
            Ok(res) => {
                repo_state.tags = res;
            }
            Err(_) => {
                repo_state.tags.clear();
            }
        }
    }

    if is_ok {
        // get current branch
        match rt.block_on(git::get_current_branch(&full_path)) {
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
        match rt.block_on(git::get_tracking_branch(&full_path)) {
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
        match rt.block_on(cmp_local_remote(input_path, repo, default_branch, true)) {
            Ok(cmp_msg) => {
                let cmp_msg = cmp_msg.to_plain_text();

                if cmp_msg.contains("not tracking")
                    || cmp_msg.contains("init commit")
                    || cmp_msg.contains("unknown revision")
                {
                    repo_state.cmp_state = StateType::Error;
                    repo_state.cmp_obj = cmp_msg;
                } else if cmp_msg.contains("already update to date.") {
                    repo_state.cmp_state = StateType::Normal;
                    let (prefix, log) = cmp_msg.split_once('.').unwrap();
                    repo_state.cmp_obj = log.trim().to_string();
                    if repo_state.cmp_obj.is_empty() {
                        repo_state.cmp_obj = prefix.to_string();
                    }
                } else {
                    repo_state.cmp_state = StateType::Warning;
                    for part in cmp_msg.split(',') {
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
    repo_state
}
