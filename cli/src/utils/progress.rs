use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use indicatif::{ProgressBar, ProgressStyle};

use mgit::core::repo::RepoId;
use mgit::utils::progress::Progress;
use mgit::utils::style_message::StyleMessage;

#[derive(Clone, Default)]
pub(crate) struct MultiProgress {
    multi_progress: Arc<Mutex<indicatif::MultiProgress>>,
    main_progress_bar: Arc<Mutex<Option<ProgressBar>>>,
    spinner_progress_bars: Arc<Mutex<HashMap<usize, ProgressBar>>>,
}

impl MultiProgress {
    fn create_total_bar(&self, total: usize) {
        let main_progress_bar = self
            .multi_progress
            .lock()
            .unwrap()
            .add(ProgressBar::new(total as u64));
        main_progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {percent}% [{bar:30.green/white}] {pos}/{len}")
                .unwrap()
                .progress_chars("=>-"),
        );
        main_progress_bar.enable_steady_tick(std::time::Duration::from_millis(500));
        let _ = self
            .main_progress_bar
            .lock()
            .unwrap()
            .insert(main_progress_bar);
    }

    fn create_progress_bar(&self, id: usize) {
        let progress_bar = self
            .multi_progress
            .lock()
            .unwrap()
            .insert(id, ProgressBar::new_spinner());
        progress_bar.set_style(
            ProgressStyle::with_template("{spinner:.green.dim.bold} {msg} ")
                .unwrap()
                .tick_chars("/-\\| "),
        );
        progress_bar.enable_steady_tick(std::time::Duration::from_millis(500));
        self.spinner_progress_bars
            .lock()
            .unwrap()
            .insert(id, progress_bar);
    }
}

impl Progress for MultiProgress {
    fn repos_start(&self, total: usize) {
        self.create_total_bar(total);
    }

    fn repos_end(&self) {
        let locked = self.main_progress_bar.lock().unwrap();
        if !locked.as_ref().unwrap().is_finished() {
            locked.as_ref().unwrap().finish();
        }
    }

    fn repo_start(&self, repo_id: RepoId) {
        self.create_progress_bar(repo_id.id);
    }

    fn repo_info(&self, repo_id: RepoId, message: StyleMessage) {
        self.spinner_progress_bars
            .lock()
            .unwrap()
            .get(&repo_id.id)
            .unwrap()
            .set_message(truncate_spinner_msg(message.to_string()));
    }

    fn repo_end(&self, repo_id: RepoId, message: StyleMessage) {
        let locked = self.spinner_progress_bars.lock().unwrap();
        let pb = locked.get(&repo_id.id).unwrap();
        if !pb.is_finished() {
            pb.finish_with_message(message.to_string());
        }

        self.main_progress_bar
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .inc(1);
    }

    fn repo_error(&self, repo_id: RepoId, message: StyleMessage) {
        self.repo_end(repo_id, message)
    }
}

pub fn get_terminal_width() -> usize {
    match term_size::dimensions() {
        Some((width, _)) => width - 10,
        _ => 70,
    }
}

pub fn truncate_spinner_msg(msg: impl AsRef<str>) -> String {
    let max_width = get_terminal_width();
    console::truncate_str(msg.as_ref(), max_width, "...").to_string()
}
