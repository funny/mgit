use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ansi_term::Colour;
use indicatif::{ProgressBar, ProgressStyle};
use mgit::utils::path::PathExtension;

use mgit::utils::progress::{Progress, RepoInfo};
use mgit::utils::style_message::StyleMessage;

#[derive(Clone, Default)]
pub(crate) struct MultiProgress {
    multi_progress: Arc<Mutex<indicatif::MultiProgress>>,
    main_progress_bar: Arc<Mutex<Option<ProgressBar>>>,
    spinner_progress_bars: Arc<Mutex<HashMap<usize, ProgressBar>>>,
    total_repos: Arc<AtomicUsize>,
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

    #[inline]
    fn prefix(idx: usize, total: usize) -> String {
        format!("[{:02}/{:02}]", idx, total)
    }

    fn spinner_start(&self, repo_info: &RepoInfo, desc: StyleMessage) -> String {
        format!(
            "{:>9} {}: {}",
            Self::prefix(repo_info.index, self.total_repos.load(Ordering::Relaxed)),
            &Colour::Purple
                .bold()
                .paint(repo_info.rel_path().display_path()),
            desc
        )
    }

    fn spinner_info(&self, repo_info: &RepoInfo, desc: StyleMessage) -> String {
        format!(
            "{:>9} {}: {}",
            Self::prefix(repo_info.index, self.total_repos.load(Ordering::Relaxed)),
            &Colour::Purple
                .bold()
                .paint(repo_info.rel_path().display_path()),
            desc
        )
    }

    fn spinner_end(&self, repo_info: &RepoInfo, status: StyleMessage, is_success: bool) -> String {
        format!(
            "{:>9} {} {}: {}",
            StyleMessage::repo_end(is_success),
            Self::prefix(repo_info.index, self.total_repos.load(Ordering::Relaxed)),
            &Colour::Green
                .bold()
                .paint(repo_info.rel_path().display_path()),
            status,
        )
    }
}

impl Progress for MultiProgress {
    fn on_batch_start(&self, total: usize) {
        self.total_repos.store(total, Ordering::Relaxed);
        self.create_total_bar(total);
    }

    fn on_batch_finish(&self) {
        let locked = self.main_progress_bar.lock().unwrap();
        if !locked.as_ref().unwrap().is_finished() {
            locked.as_ref().unwrap().finish();
            tracing::info!("");
        }
    }

    fn on_repo_start(&self, repo_info: &RepoInfo, message: StyleMessage) {
        self.create_progress_bar(repo_info.index);
        self.spinner_progress_bars
            .lock()
            .unwrap()
            .get(&repo_info.index)
            .unwrap()
            .set_message(truncate_spinner_msg(self.spinner_start(repo_info, message)));
    }

    fn on_repo_update(&self, repo_info: &RepoInfo, message: StyleMessage) {
        self.spinner_progress_bars
            .lock()
            .unwrap()
            .get(&repo_info.index)
            .unwrap()
            .set_message(truncate_spinner_msg(self.spinner_info(repo_info, message)));
    }

    fn on_repo_success(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let locked = self.spinner_progress_bars.lock().unwrap();
        let pb = locked.get(&repo_info.index).unwrap();
        if !pb.is_finished() {
            pb.finish_with_message(truncate_spinner_msg(
                self.spinner_end(repo_info, message, true),
            ));
        }

        self.main_progress_bar
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .inc(1);
    }

    fn on_repo_error(&self, repo_info: &RepoInfo, message: StyleMessage) {
        let locked = self.spinner_progress_bars.lock().unwrap();
        let pb = locked.get(&repo_info.index).unwrap();
        if !pb.is_finished() {
            pb.finish_with_message(self.spinner_end(repo_info, message, false));
        }

        self.main_progress_bar
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .inc(1);
    }
}

pub fn get_terminal_width() -> usize {
    match console::Term::stdout().size() {
        (_, width) if width > 10 => width as usize - 10,
        _ => 70,
    }
}

pub fn truncate_spinner_msg(msg: impl AsRef<str>) -> String {
    let max_width = get_terminal_width();
    console::truncate_str(msg.as_ref(), max_width, "...").to_string()
}
