use mgit::core::repo::TomlRepo;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::editor::{get_repo_state, CommandType, RepoMessage};
use crate::logger::LOG_DIR;
use mgit::utils::progress::{Progress, RepoInfo};
use mgit::utils::style_message::StyleMessage;

#[derive(Debug, Clone)]
pub(crate) struct OpsMessageCollector {
    repo_state_buffers: Vec<Arc<Mutex<StyleMessage>>>,
    file_loggers: Vec<Arc<Mutex<File>>>,
    repo_names: Vec<String>,
    sender: Arc<Mutex<Sender<RepoMessage>>>,
    pub progress: Arc<AtomicUsize>,
    pub command_type: CommandType,
    pub project_path: String,
    pub default_branch: Option<String>,
}

impl OpsMessageCollector {
    pub(crate) fn new(sender: Sender<RepoMessage>, progress: Arc<AtomicUsize>) -> Self {
        Self {
            repo_state_buffers: vec![],
            file_loggers: vec![],
            repo_names: vec![],
            sender: Arc::new(Mutex::new(sender)),
            command_type: CommandType::None,
            project_path: String::new(),
            default_branch: None,
            progress,
        }
    }

    pub(crate) fn update(&mut self, toml_repos: &Vec<TomlRepo>) {
        let repo_state_buffers = (0..toml_repos.len())
            .map(|_| Arc::new(Mutex::new(StyleMessage::default())))
            .collect();

        let repo_names = toml_repos
            .iter()
            .enumerate()
            .map(|(i, toml)| Self::generate_repo_name(i, toml))
            .collect::<Vec<_>>();

        let file_loggers = toml_repos
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let file_name = &repo_names[i];
                let file_path = LOG_DIR.join(file_name);
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

    pub fn read_ops_message(&self, id: usize) -> StyleMessage {
        self.repo_state_buffers[id].lock().unwrap().deref().clone()
    }

    #[rustfmt::skip]
    pub fn generate_repo_name(id: usize, toml_repo: &TomlRepo) -> String {
        let regex = Regex::new(r#"[^a-zA-Z0-9]+"#).unwrap();
        format!(
            "{:02}-{}-{}",
            id,
            regex.replace_all(toml_repo.local.as_ref().unwrap_or(&"no_local".to_string()), "_"),
            regex.replace_all(toml_repo.remote.as_ref().unwrap_or(&"no_remote".to_string()), "_"),
        )
    }
}

impl Progress for OpsMessageCollector {
    fn repos_start(&self, _total: usize) {
        self.progress.store(0, Ordering::Relaxed);
    }

    fn repos_end(&self) {}

    fn repo_start(&self, repo_info: &RepoInfo, message: StyleMessage) {
        {
            let mut file = self.file_loggers[repo_info.id].lock().unwrap();
            writeln!(
                file,
                "**********start repo: {}**********",
                self.repo_names[repo_info.id]
            )
            .unwrap();
        }

        self.repo_info(repo_info, message);
    }

    fn repo_info(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let ptr = self.repo_state_buffers[repo_info.id].clone();
        ptr.lock().unwrap().replace(message.clone());

        let mut file = self.file_loggers[repo_info.id].lock().unwrap();
        writeln!(file, "{}", message.to_plain_text()).unwrap();
    }

    #[allow(unused_variables)]
    fn repo_end(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let sender = self.sender.clone();
        let id = repo_info.id;
        let toml_repo = repo_info.toml_repo.clone();
        let project_path = self.project_path.clone();
        let default_branch = self.default_branch.clone();
        let progress = self.progress.clone();
        thread::spawn(move || {
            #[cfg(feature = "dev")]
            {
                thread::sleep(std::time::Duration::from_millis(100 * id as u64));
            }
            progress.fetch_add(1, Ordering::Relaxed);
            let repo_state = get_repo_state(&toml_repo, &project_path, &default_branch);
            sender
                .lock()
                .unwrap()
                .send(RepoMessage::new(CommandType::None, repo_state, Some(id)))
                .unwrap();
        });

        let mut file = self.file_loggers[repo_info.id].lock().unwrap();
        writeln!(
            file,
            "==========end repo: {}==========",
            self.repo_names[repo_info.id]
        )
        .unwrap();
    }

    fn repo_error(&self, repo_info: &RepoInfo, message: StyleMessage) {
        self.repo_end(repo_info, message)
    }
}
