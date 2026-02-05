use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use regex::Regex;
use tracing::{debug, error, info};

use mgit::config::RepoConfig;
use mgit::utils::progress::{Progress, RepoInfo};
use mgit::utils::style_message::StyleMessage;

use crate::app::events::CommandType;
use crate::app::events::{BackendEvent, Event};
use crate::app::repo_manager::get_repo_state;
use crate::utils::logger::log_dir;

#[derive(Debug, Clone)]
pub(crate) struct OpsMessageCollector {
    repo_state_buffers: Vec<Arc<Mutex<StyleMessage>>>,
    file_loggers: Vec<Arc<Mutex<File>>>,
    repo_names: Vec<String>,
    sender: Arc<Mutex<Sender<Event>>>,
    pub progress: Arc<AtomicUsize>,
    pub command_type: CommandType,
    pub run_id: u64,
    batch_started_at: Arc<Mutex<Option<Instant>>>,
    batch_finished_sent: Arc<AtomicBool>,
    pub project_path: String,
    pub default_branch: Option<String>,
}

impl OpsMessageCollector {
    pub(crate) fn new(sender: Sender<Event>, progress: Arc<AtomicUsize>) -> Self {
        Self {
            repo_state_buffers: vec![],
            file_loggers: vec![],
            repo_names: vec![],
            sender: Arc::new(Mutex::new(sender)),
            command_type: CommandType::None,
            run_id: 0,
            batch_started_at: Arc::new(Mutex::new(None)),
            batch_finished_sent: Arc::new(AtomicBool::new(false)),
            project_path: String::new(),
            default_branch: None,
            progress,
        }
    }

    pub(crate) fn send_command_finished_once(&self) {
        if self.command_type == CommandType::None {
            return;
        }
        if self.batch_finished_sent.swap(true, Ordering::AcqRel) {
            return;
        }
        let _ = self
            .sender
            .lock()
            .unwrap()
            .send(Event::Backend(BackendEvent::CommandFinished {
                run_id: self.run_id,
                command: self.command_type.into(),
            }));
    }

    pub(crate) fn update(&mut self, repo_configs: &Vec<RepoConfig>) {
        let repo_state_buffers = (0..repo_configs.len())
            .map(|_| Arc::new(Mutex::new(StyleMessage::default())))
            .collect();

        let repo_names = repo_configs
            .iter()
            .enumerate()
            .map(|(i, toml)| Self::generate_repo_name(i, toml))
            .collect::<Vec<_>>();

        let file_loggers = repo_configs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let file_name = &repo_names[i];
                let file_path = log_dir().join(format!("{}.txt", file_name));
                let file = if !file_path.exists() {
                    File::create(file_path).unwrap()
                } else {
                    OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(file_path)
                        .unwrap()
                };
                Arc::new(Mutex::new(file))
            })
            .collect();

        self.repo_state_buffers = repo_state_buffers;
        self.file_loggers = file_loggers;
        self.repo_names = repo_names;
    }

    pub fn read_ops_message(&self, idx: usize) -> Vec<StyleMessage> {
        if idx >= self.repo_state_buffers.len() {
            return Vec::new();
        }
        let buffer = self.repo_state_buffers[idx].lock().unwrap();
        vec![buffer.clone()]
    }

    #[rustfmt::skip]
    pub fn generate_repo_name(id: usize, repo_config: &RepoConfig) -> String {
        let regex = Regex::new(r#"[^a-zA-Z0-9]+"#).unwrap();
        format!(
            "{:02}-{}-{}",
            id,
            regex.replace_all(repo_config.local.as_ref().unwrap_or(&"no_local".to_string()), "_"),
            regex.replace_all(repo_config.remote.as_ref().unwrap_or(&"no_remote".to_string()), "_"),
        )
    }
}

impl Progress for OpsMessageCollector {
    fn on_batch_start(&self, _total: usize) {
        self.progress.store(0, Ordering::Relaxed);
        self.batch_finished_sent.store(false, Ordering::Relaxed);
        *self.batch_started_at.lock().unwrap() = Some(Instant::now());
        info!(
            run_id = self.run_id,
            command = ?self.command_type,
            total = _total,
            "ops_batch_start"
        );
    }

    fn on_batch_finish(&self) {
        let duration_ms = self
            .batch_started_at
            .lock()
            .unwrap()
            .take()
            .map(|t| t.elapsed().as_millis());
        debug!(
            run_id = self.run_id,
            command = ?self.command_type,
            duration_ms,
            "ops_batch_finish"
        );
        self.send_command_finished_once();
    }

    fn on_repo_start(&self, repo_info: &RepoInfo, message: StyleMessage) {
        {
            let mut file = self.file_loggers[repo_info.id].lock().unwrap();
            writeln!(
                file,
                "**********start repo: {}**********",
                self.repo_names[repo_info.id]
            )
            .unwrap();
        }

        debug!(
            run_id = self.run_id,
            command = ?self.command_type,
            repo_id = repo_info.id,
            repo_rel_path = repo_info.rel_path(),
            "ops_repo_start"
        );
        self.on_repo_update(repo_info, message);
    }

    fn on_repo_update(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let ptr = self.repo_state_buffers[repo_info.id].clone();
        ptr.lock().unwrap().replace(message.clone());

        let mut file = self.file_loggers[repo_info.id].lock().unwrap();
        writeln!(file, "{}", message.to_plain_text()).unwrap();
    }

    #[allow(unused_variables)]
    fn on_repo_success(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let sender = self.sender.clone();
        let id = repo_info.id;
        let repo_config = repo_info.repo_config.clone();
        let project_path = self.project_path.clone();
        let default_branch = self.default_branch.clone();
        let progress = self.progress.clone();
        let run_id = self.run_id;
        debug!(
            run_id,
            command = ?self.command_type,
            repo_id = repo_info.id,
            repo_rel_path = repo_info.rel_path(),
            "ops_repo_finished"
        );
        thread::spawn(move || {
            #[cfg(feature = "dev")]
            {
                thread::sleep(std::time::Duration::from_millis(100 * id as u64));
            }
            progress.fetch_add(1, Ordering::Relaxed);
            let repo_state = get_repo_state(&repo_config, &project_path, &default_branch);
            sender
                .lock()
                .unwrap()
                .send(Event::Backend(BackendEvent::RepoStateUpdated {
                    run_id,
                    id,
                    repo_state,
                }))
                .unwrap();
        });

        let mut file = self.file_loggers[repo_info.id].lock().unwrap();
        writeln!(file, "{}", self.repo_names[repo_info.id]).unwrap();
    }

    fn on_repo_error(&self, repo_info: &RepoInfo, message: StyleMessage) {
        error!(
            run_id = self.run_id,
            command = ?self.command_type,
            repo_id = repo_info.id,
            repo_rel_path = repo_info.rel_path(),
            message = message.to_plain_text(),
            "ops_repo_error"
        );
        self.on_repo_success(repo_info, message)
    }
}
