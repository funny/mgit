use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::editor::RepoMessages;
use crate::logger::LOG_DIR;
use mgit::utils::progress::{Progress, RepoInfo};
use mgit::utils::style_message::StyleMessage;

#[derive(Debug, Clone)]
pub(crate) struct RepoMessageCollector {
    repo_state_buffers: Vec<Arc<Mutex<StyleMessage>>>,
    file_loggers: Vec<Arc<Mutex<File>>>,
    repo_names: Vec<String>,
}

impl RepoMessageCollector {
    pub(crate) fn new(repo_messages: &RepoMessages) -> Self {
        let repo_state_buffers = repo_messages
            .iter()
            .map(|msg| msg.message.clone())
            .collect();

        let repo_names = repo_messages
            .iter()
            .map(|msg| msg.generate_repo_name())
            .collect::<Vec<_>>();

        let file_loggers = repo_messages
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

        Self {
            repo_state_buffers,
            file_loggers,
            repo_names,
        }
    }
}

impl Progress for RepoMessageCollector {
    fn repos_start(&self, _total: usize) {
        // do nothing
    }

    fn repos_end(&self) {
        // do noting
    }

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

    fn repo_end(&self, repo_info: &RepoInfo, message: StyleMessage) {
        self.repo_info(repo_info, message);

        let mut file = self.file_loggers[repo_info.id].lock().unwrap();
        writeln!(
            file,
            "==========end repo: {}==========",
            self.repo_names[repo_info.id]
        )
        .unwrap();
    }

    fn repo_error(&self, repo_info: &RepoInfo, message: StyleMessage) {
        self.repo_info(repo_info, message)
    }
}
