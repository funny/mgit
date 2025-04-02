use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use mgit::utils::logger::Log;
use mgit::utils::path::PathExtension;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use mgit::core::git;
use mgit::core::repo::{cmp_local_remote, TomlRepo};
use mgit::ops;
use mgit::ops::{
    CleanOptions, FetchOptions, InitOptions, SnapshotOptions, SnapshotType, SyncOptions,
    TrackOptions,
};
use mgit::utils::progress::Progress;

use crate::editor::Editor;
use crate::toml_settings::SyncType;
use crate::utils::command::CommandType;
use crate::utils::logger::GUI_LOGGER;

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

#[derive(Debug, Clone)]
pub struct RepoMessage {
    pub id: Option<usize>,
    pub command_type: CommandType,
    pub repo_state: RepoState,
}

impl RepoMessage {
    pub fn new(command_type: CommandType, repo_state: RepoState, id: Option<usize>) -> Self {
        Self {
            id,
            command_type,
            repo_state,
        }
    }
}

impl Editor {
    pub(crate) fn exec_ops(&mut self, command_type: CommandType) {
        // to show in ui
        if self.config_file.is_empty() {
            self.config_file = format!("{}/.gitrepos", &self.project_path);
        }

        self.progress = Arc::new(AtomicUsize::new(0));
        match command_type {
            CommandType::Init => {
                self.config_file = format!("{}/.gitrepos", &self.project_path);

                let path = Some(&self.project_path);
                let force = self.toml_user_settings.init_force;

                let options = InitOptions::new(path, force);
                let send = self.send.clone();
                self.clear_status();
                std::thread::spawn(move || {
                    let _ = ops::init_repo(options);
                    send.send(RepoMessage::new(command_type, RepoState::default(), None))
                        .unwrap();
                });
            }

            CommandType::Snapshot => {
                let path = Some(&self.project_path);
                let config_path = Some(&self.config_file);
                let snapshot_type = self
                    .toml_user_settings
                    .snapshot_branch
                    .and_then(|b| match b {
                        true => Some(SnapshotType::Branch),
                        false => None,
                    });
                let force = self.toml_user_settings.snapshot_force;
                let ignore: Option<Vec<String>> = self
                    .get_snapshot_ignore()
                    .map(|content| content.split('\n').map(|s| s.to_string()).collect());

                let options = SnapshotOptions::new(path, config_path, force, snapshot_type, ignore);
                let send = self.send.clone();

                self.push_recent_config();
                self.clear_status();

                std::thread::spawn(move || {
                    let _ = ops::snapshot_repo(options);
                    send.send(RepoMessage::new(command_type, RepoState::default(), None))
                        .unwrap();
                });
            }

            CommandType::Fetch => {
                let path = Some(&self.project_path);
                let config_path = Some(&self.config_file);
                let thread = self.toml_user_settings.sync_thread.map(|t| t as usize);
                let depth = self.toml_user_settings.sync_depth.map(|d| d as usize);
                let ignore: Option<Vec<String>> = self.get_ignores();
                let labels = self.get_labels();
                let silent = Some(true);

                let options =
                    FetchOptions::new(path, config_path, thread, silent, depth, ignore, labels);

                self.reset_repo_state(StateType::Updating);
                let progress = self.progress(command_type);
                std::thread::spawn(move || {
                    let _ = ops::fetch_repos(options, progress);
                });
            }

            CommandType::Sync | CommandType::SyncHard => {
                let path = Some(&self.project_path);
                let config_path = Some(&self.config_file);
                // check if command_type is CommandType::SyncHard
                let sync_type = match command_type == CommandType::SyncHard {
                    true => SyncType::Hard,
                    false => self.toml_user_settings.sync_type.unwrap_or(SyncType::Stash),
                };
                // option none or --stash or --hard
                let (hard, stash) = match sync_type {
                    SyncType::Normal => (Some(false), Some(false)),
                    SyncType::Stash => (Some(false), Some(true)),
                    SyncType::Hard => (Some(true), Some(false)),
                };
                // option --no-checkout
                let no_checkout = self.toml_user_settings.sync_no_checkout;
                // option --no-track
                let no_track = self.toml_user_settings.sync_no_track;
                // option --thread <num>
                let thread_count = self.toml_user_settings.sync_thread.map(|t| t as usize);
                // option --depth <num>
                let depth = self.toml_user_settings.sync_depth.map(|d| d as usize);
                // option --ignore
                let ignore: Option<Vec<String>> = self.get_ignores();
                // option --labels
                let labels = self.get_labels();
                // option --silent
                let silent = Some(true);

                let options = SyncOptions::new(
                    path,
                    config_path,
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
                let progress = self.progress(command_type);

                std::thread::spawn(move || {
                    let _ = ops::sync_repo(options, progress);
                });
            }

            CommandType::Track => {
                let path = Some(&self.project_path);
                let config_path = Some(&self.config_file);
                let ignore: Option<Vec<String>> = self.get_ignores();

                let options = TrackOptions::new(path, config_path, ignore);

                self.reset_repo_state(StateType::Updating);
                let progress = self.progress(command_type);

                std::thread::spawn(move || {
                    let _ = ops::track(options, progress);
                });
            }

            CommandType::Clean => {
                let path = Some(&self.project_path);
                let config_path = Some(&self.config_file);
                // option --labels
                let labels = self.get_labels();

                let options = CleanOptions::new(path, config_path, labels);
                let send = self.send.clone();

                self.reset_repo_state(StateType::Updating);

                std::thread::spawn(move || {
                    let _ = ops::clean_repo(options);
                    send.send(RepoMessage::new(command_type, RepoState::default(), None))
                        .unwrap();
                });
            }

            CommandType::Refresh => {
                self.progress.store(0, Ordering::Relaxed);
                self.clear_status();
                self.load_config();
                self.reset_repo_state(StateType::Updating);
                self.get_repo_states();
            }

            CommandType::NewBranch => {
                let options = self.new_branch_window.get_options();

                if let Some(path) = options.new_config_path.as_ref() {
                    self.config_file = path.norm_path();
                    self.push_recent_config();
                }

                let send = self.send.clone();
                self.clear_status();

                std::thread::spawn(move || {
                    if let Err(e) = ops::new_remote_branch(options) {
                        GUI_LOGGER.error(e.to_string().into());
                    }

                    send.send(RepoMessage::new(command_type, RepoState::default(), None))
                        .unwrap();
                });
            }

            CommandType::NewTag => {
                let options = self.new_tag_window.get_options();
                println!("exec new tag: {:?}", options);

                let send = self.send.clone();
                self.clear_status();

                std::thread::spawn(move || {
                    if let Err(e) = ops::new_tag(options) {
                        GUI_LOGGER.error(e.to_string().into());
                    }

                    send.send(RepoMessage::new(command_type, RepoState::default(), None))
                        .unwrap();
                });
            }

            CommandType::None => {}
        }
    }

    pub(crate) fn handle_channel_recv(&mut self) {
        // as callback after execute command
        if let Ok(repo_message) = self.recv.try_recv() {
            if repo_message.command_type == CommandType::None {
                if let Some(id) = repo_message.id {
                    self.repo_states[id] = RepoState {
                        no_ignore: self.repo_states[id].no_ignore,
                        disable_by_label: self.repo_states[id].disable_by_label,
                        ..repo_message.repo_state
                    };
                };
            } else {
                self.load_config();
                self.reset_repo_state(StateType::Updating);
                self.get_repo_states();
            }

            self.context.request_repaint();
        }
    }

    fn progress(&mut self, command_type: CommandType) -> impl Progress {
        self.ops_message_collector.progress = self.progress.clone();
        self.ops_message_collector.command_type = command_type;
        self.ops_message_collector.project_path = self.project_path.clone();
        self.ops_message_collector.default_branch = self.toml_config.default_remote.clone();
        self.ops_message_collector.clone()
    }

    fn clear_status(&mut self) {
        self.clear_repo_state();
        self.clear_toml_config();
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

    pub(crate) fn get_repo_states(&mut self) {
        // get repository state
        if let Some(repos) = &self.toml_config.repos.clone() {
            let project_path = self.project_path.clone();
            let default_branch = self.toml_config.default_branch.clone();
            get_repo_states_parallel(
                repos.to_owned(),
                project_path,
                default_branch,
                self.send.clone(),
                self.context.clone(),
            )
        }
    }
}

fn get_repo_states_parallel(
    toml_repos: Vec<TomlRepo>,
    project_path: String,
    default_branch: Option<String>,
    sender: Sender<RepoMessage>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let thread_pool = match rayon::ThreadPoolBuilder::new().build() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let sender = Arc::new(Mutex::new(sender));
        thread_pool.install(|| {
            toml_repos
                .iter()
                .enumerate()
                .collect::<Vec<_>>()
                .par_iter()
                .for_each_with(&sender, |s, (id, repo)| {
                    let repo_state = get_repo_state(repo, &project_path, &default_branch);
                    s.lock()
                        .unwrap()
                        .send(RepoMessage::new(CommandType::None, repo_state, Some(*id)))
                        .unwrap();
                })
        });

        // NOTE: 保证所有仓库忽略后正常渲染
        ctx.request_repaint();
    });
}

pub(crate) fn get_repo_state(
    repo: &TomlRepo,
    project_path: &String,
    default_branch: &Option<String>,
) -> RepoState {
    let mut repo_state = RepoState::default();
    let input_path = Path::new(&project_path);
    let full_path = input_path.join(repo.to_owned().local.unwrap());

    let mut is_ok = true;
    if let Err(e) = git::is_repository(&full_path) {
        repo_state.err_msg = e.to_string();
        is_ok = false;
    }

    if let Err(e) = git::has_authenticity(&full_path) {
        repo_state.err_msg = e.to_string();
        is_ok = false;
    }

    if is_ok {
        // get tags
        match git::get_head_tags(&full_path) {
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
        match git::get_current_branch(&full_path) {
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
        match git::get_tracking_branch(&full_path) {
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
        match cmp_local_remote(input_path, repo, default_branch, true) {
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
