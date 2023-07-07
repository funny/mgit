use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::editor::RepoMessages;
use crate::logger::LOG_DIR;
use mgit::core::repo::RepoId;
use mgit::utils::progress::Progress;
use mgit::utils::style_message::StyleMessage;

#[derive(Debug, Clone)]
pub(crate) struct RepoMessageCollector {
    repo_state_buffers: HashMap<String, Arc<Mutex<StyleMessage>>>,
    file_loggers: HashMap<String, Arc<Mutex<File>>>,
}

impl RepoMessageCollector {
    pub(crate) fn new(repo_messages: &RepoMessages) -> Self {
        let repo_state_buffers = repo_messages
            .iter()
            .map(|(repo_name, msg)| (repo_name.clone(), msg.clone()))
            .collect();

        let file_loggers = repo_messages
            .iter()
            .map(|(repo_name, _)| {
                (
                    repo_name.clone(),
                    Arc::new(Mutex::new(
                        File::create(LOG_DIR.join(repo_name.as_str())).unwrap(),
                    )),
                )
            })
            .collect();

        Self {
            repo_state_buffers,
            file_loggers,
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

    fn repo_start(&self, repo_id: RepoId) {
        let mut file = self.file_loggers[&repo_id.repo].lock().unwrap();
        file.write_fmt(format_args!(
            "**********start repo: {}**********\n",
            repo_id.repo
        ))
        .unwrap();
    }

    fn repo_info(&self, repo_id: RepoId, message: StyleMessage) {
        let ptr = self.repo_state_buffers[&repo_id.repo].clone();
        ptr.lock().unwrap().replace(message.clone());

        let mut file = self.file_loggers[&repo_id.repo].lock().unwrap();
        file.write_fmt(format_args!("{}\n", message.to_plain_text()))
            .unwrap();
    }

    fn repo_end(&self, repo_id: RepoId, message: StyleMessage) {
        self.repo_info(repo_id.clone(), message);

        let mut file = self.file_loggers[&repo_id.repo].lock().unwrap();
        file.write_fmt(format_args!(
            "==========end repo: {}==========\n",
            repo_id.repo
        ))
        .unwrap();
    }

    fn repo_error(&self, repo_id: RepoId, message: StyleMessage) {
        self.repo_info(repo_id, message)
    }
}
