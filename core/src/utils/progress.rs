use crate::core::repo::RepoId;
use crate::utils::style_message::StyleMessage;

pub trait Progress: Send + Sync + Clone {
    /// set total repo count, all repositories will execute in parallel
    fn repos_start(&self, total: usize);

    /// notify total repo finished
    fn repos_end(&self);

    /// repo start
    fn repo_start(&self, repo_id: RepoId);

    /// repo info message
    fn repo_info(&self, repo_id: RepoId, message: StyleMessage);

    /// repo error message
    fn repo_error(&self, repo_id: RepoId, message: StyleMessage);

    /// repo success ended with message
    fn repo_end(&self, repo_id: RepoId, message: StyleMessage);
}
