use lazy_static::lazy_static;
use mgit::core::repo::RepoId;
use mgit::utils::progress::Progress;
use mgit::utils::style_message::StyleMessage;
use std::str::FromStr;
use std::{path::PathBuf, process::Stdio};

#[allow(unused)]
pub const DEFAULT_BRANCH: &str = "master";

#[allow(unused)]
pub mod failed_message {
    pub const GIT_INIT: &str = "git init failed";
    pub const GIT_ADD_REMOTE: &str = "git add remote failed";
    pub const GIT_STAGE: &str = "git stage failed";
    pub const GIT_COMMIT: &str = "git commit failed";
    pub const GIT_STATUS: &str = "git status failed";
    pub const GIT_CHECKOUT: &str = "git checkout failed";
    pub const GIT_RESET: &str = "git reset failed";
    pub const GIT_STASH_LIST: &str = "git stash list failed";
    pub const GIT_STASH_POP: &str = "git stash pop failed";
    pub const GIT_BRANCH: &str = "git branch failed";
    pub const GIT_FETCH: &str = "git fetch failed";
    pub const GIT_CONFIG: &str = "git config failed";
    pub const GIT_REV_LIST: &str = "git rev-list failed";

    pub const WRITE_FILE: &str = "write file failed";
}

pub fn exec_cmd(path: &PathBuf, cmd: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let output = std::process::Command::new(cmd)
        .current_dir(path.to_path_buf())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    match output.status.success() {
        false => Err(anyhow::anyhow!(stderr)),
        true => Ok(stdout),
    }
}

lazy_static! {
    static ref USE_GITEA: bool = use_gitea();
    pub static ref MGIT_REPO: &'static str = match &USE_GITEA as &bool {
        true => "http://localhost:3000/mgit/mgit.git",
        false => "https://github.com/funny/mgit.git",
    };
    pub static ref IMGUI_REPO: &'static str = match &USE_GITEA as &bool {
        true => "http://localhost:3000/mgit/imgui-rs.git",
        false => "https://github.com/imgui-rs/imgui-rs.git",
    };
    pub static ref SBERT_REPO: &'static str = match &USE_GITEA as &bool {
        true => "http://localhost:3000/mgit/rust-sbert.git",
        false => "https://gitee.com/icze1i0n/rust-sbert.git",
    };
    pub static ref CSBOOKS_REPO: &'static str = match &USE_GITEA as &bool {
        true => "http://localhost:3000/mgit/CS-Books.git",
        false => "https://gitee.com/ForthEspada/CS-Books.git",
    };
}

pub struct TomlBuilder {
    toml_string: String,
}

impl TomlBuilder {
    pub fn new() -> Self {
        TomlBuilder {
            toml_string:
                "# This file is automatically @generated by mgit.\n# Editing it as you wish.\n"
                    .to_string(),
        }
    }

    pub fn build(self) -> String {
        self.toml_string
    }

    pub fn default_branch(mut self, default_branch: impl AsRef<str>) -> Self {
        self.toml_string.push_str(&format!(
            "default-branch = \"{}\"\n",
            default_branch.as_ref()
        ));
        self
    }

    pub fn join_repo(
        mut self,
        local: &str,
        remote: &str,
        branch: Option<&str>,
        commit: Option<&str>,
        tag: Option<&str>,
    ) -> Self {
        self.toml_string
            .push_str(&format!("\n[[repos]]\nlocal = \"{}\"\n", local));
        self.toml_string
            .push_str(&format!("remote = \"{}\"\n", remote));
        if let Some(branch) = branch {
            self.toml_string
                .push_str(&format!("branch = \"{}\"\n", branch));
        }
        if let Some(commit) = commit {
            self.toml_string
                .push_str(&format!("commit = \"{}\"\n", commit));
        }
        if let Some(tag) = tag {
            self.toml_string.push_str(&format!("tag = \"{}\"\n", tag));
        }
        self
    }
}

fn use_gitea() -> bool {
    #[cfg(feature = "use_gitea")]
    {
        true
    }
    #[cfg(not(feature = "use_gitea"))]
    {
        false
    }
}

pub fn retry<T>(
    times: usize,
    sleep: std::time::Duration,
    f: impl Fn() -> Result<T, anyhow::Error>,
) -> Result<T, anyhow::Error> {
    let mut result = None::<Result<T, anyhow::Error>>;
    for i in 0..times {
        match f() {
            Ok(r) => {
                result = Some(Ok(r));
                break;
            }
            Err(e) => {
                println!("retry[{}]: {}", i, e);
                result = Some(Err(e));
                std::thread::sleep(sleep);
            }
        }
    }
    result.unwrap()
}

#[derive(Clone, Default)]
pub struct TestProgress;

impl Progress for TestProgress {
    fn repos_start(&self, _total: usize) {}

    fn repos_end(&self) {}

    fn repo_start(&self, _repo_id: RepoId) {}

    fn repo_info(&self, _repo_id: RepoId, _message: StyleMessage) {}

    fn repo_error(&self, _repo_id: RepoId, _message: StyleMessage) {}

    fn repo_end(&self, _repo_id: RepoId, _message: StyleMessage) {}
}
