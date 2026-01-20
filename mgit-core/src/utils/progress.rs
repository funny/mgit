use crate::config::RepoConfig;
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Clone)]
pub struct RepoInfo<'a> {
    pub id: usize,
    pub index: usize,
    pub repo_config: &'a RepoConfig,
}

impl<'a> RepoInfo<'a> {
    pub fn new(id: usize, index: usize, repo_config: &'a RepoConfig) -> Self {
        Self {
            id,
            index,
            repo_config,
        }
    }

    #[inline]
    pub fn rel_path(&self) -> &str {
        self.repo_config.local.as_ref().unwrap()
    }
}

pub trait Progress: Send + Sync + Clone {
    /// set total repo count, all repositories will execute in parallel
    fn on_batch_start(&self, total: usize);

    /// notify total repo finished
    fn on_batch_finish(&self);

    /// repo start
    fn on_repo_start(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo info message
    fn on_repo_update(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo success ended with message
    fn on_repo_success(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo error message
    fn on_repo_error(&self, repo_info: &RepoInfo, message: StyleMessage);
}
