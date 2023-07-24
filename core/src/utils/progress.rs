use crate::core::repo::TomlRepo;
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Clone)]
pub struct RepoInfo<'a> {
    pub id: usize,
    pub index: usize,
    pub toml_repo: &'a TomlRepo,
}

impl<'a> RepoInfo<'a> {
    pub fn new(id: usize, index: usize, toml_repo: &'a TomlRepo) -> Self {
        Self {
            id,
            index,
            toml_repo,
        }
    }

    #[inline]
    pub fn rel_path(&self) -> &str {
        self.toml_repo.local.as_ref().unwrap()
    }
}

pub trait Progress: Send + Sync + Clone {
    /// set total repo count, all repositories will execute in parallel
    fn repos_start(&self, total: usize);

    /// notify total repo finished
    fn repos_end(&self);

    /// repo start
    fn repo_start(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo info message
    fn repo_info(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo success ended with message
    fn repo_end(&self, repo_info: &RepoInfo, message: StyleMessage);

    /// repo error message
    fn repo_error(&self, repo_info: &RepoInfo, message: StyleMessage);
}
