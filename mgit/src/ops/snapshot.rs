use globset::GlobBuilder;

use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::MgitConfig;
use crate::config::RepoConfig;
use crate::error::MgitResult;
use crate::git;

use crate::utils::path::PathExtension;
use crate::utils::style_message::StyleMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SnapshotType {
    Commit,
    Branch,
}

pub struct SnapshotOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub force: bool,
    pub snapshot_type: SnapshotType,
    pub ignore: Option<Vec<String>>,
}

impl SnapshotOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        force: Option<bool>,
        snapshot_type: Option<SnapshotType>,
        ignore: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            force: force.unwrap_or(false),
            snapshot_type: snapshot_type.unwrap_or(SnapshotType::Commit),
            ignore,
        }
    }
}

pub async fn snapshot_repo(options: SnapshotOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;
    let force = options.force;
    let snapshot_type = &options.snapshot_type;
    let ignore = options.ignore.clone(); // Clone for closure

    tracing::info!(message = %StyleMessage::ops_start("take snapshot", path));

    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        return Err(crate::error::MgitError::DirNotFound { path: path.clone() });
    }

    // check if .gitrepos exists
    if config_path.is_file() && !force {
        return Err(crate::error::MgitError::DirAlreadyInited { path: path.clone() });
    }

    let mut mgit_config = MgitConfig {
        version: None,
        default_branch: Some(String::from("develop")),
        default_remote: None,
        repos: None,
    };

    // search for git repos and create .gitrepos file
    let glob = GlobBuilder::new("**/.git")
        .literal_separator(true)
        .build()
        .unwrap()
        .compile_matcher();

    tracing::info!("search and add git repos:");

    let input_path = path.to_owned();

    // Scan repos in blocking thread since WalkDir and many small git ops are blocking-heavy
    // Alternatively, we could walkdir and spawn async tasks for git ops, but snapshot is usually fast enough
    let (repos, file_count) = tokio::task::spawn_blocking(move || {
        // Need a runtime handle to block_on async git calls if we use async git here.
        // Or we can assume git ops in snapshot are light enough or rewrite them to blocking?
        // Wait, git::is_repository is async. We can't call async from blocking thread easily without block_on.
        // It's better to collect paths first, then process them async.

        let mut paths = Vec::new();
        let mut file_count = 0;
        let mut it = WalkDir::new(&input_path).into_iter();

        loop {
            let entry = match it.next() {
                None => break,
                Some(Err(err)) => panic!("ERROR: {}", err),
                Some(Ok(entry)) => entry,
            };
            let path = entry.path();

            if glob.is_match(path) {
                let mut pb = path.to_path_buf();
                pb.pop();
                paths.push(pb);
                it.skip_current_dir();
                continue;
            }
            file_count += 1;
        }
        (paths, file_count)
    })
    .await
    .map_err(|e| crate::error::MgitError::OpsError {
        message: format!("Failed to walk directory: {}", e),
    })?;

    let mut final_repos: Vec<RepoConfig> = Vec::new();

    for pb in repos {
        let rel_path = pb.strip_prefix(path).unwrap();
        let norm_path = rel_path.norm_path();
        let norm_str = norm_path.display_path();

        // ignore specified path
        if matches!(&ignore, Some(paths) if paths.contains(&norm_str)) {
            continue;
        }

        // check repository valid
        if git::is_repository(pb.as_path()).await.is_err() {
            tracing::error!("Failed to open repo {}!", &norm_str);
            continue;
        }

        // get remote
        let remote = git::find_remote_url_by_name(&pb, "origin").await.ok();
        let mut commit: Option<String> = None;
        let mut branch: Option<String> = None;

        match snapshot_type {
            SnapshotType::Commit => {
                if let Ok(oid) = git::get_current_commit(pb.as_path()).await {
                    commit = Some(oid);
                }
            }
            SnapshotType::Branch => {
                if let Ok(refname) = git::get_tracking_branch(pb.as_path()).await {
                    if let Some((_, branch_ref)) = refname.split_once('/') {
                        branch = Some(branch_ref.trim().to_string());
                    }
                }
            }
        }

        let sparse = match git::sparse_checkout_list(pb.as_path()).await {
            Err(_) => None,
            Ok(content) if content.trim().is_empty() => None,
            Ok(content) => {
                let list: Vec<_> = content.trim().lines().map(|s| s.to_string()).collect();
                Some(list)
            }
        };

        let repo_config = RepoConfig {
            local: Some(norm_str.clone()),
            remote,
            branch,
            tag: None,
            commit,
            sparse,
            labels: None,
        };
        final_repos.push(repo_config);
        tracing::info!("  + {}", norm_str);
    }

    final_repos.sort_by(|a, b| {
        a.local
            .as_ref()
            .unwrap()
            .to_lowercase()
            .cmp(&b.local.as_ref().unwrap().to_lowercase())
    });

    let repo_count = final_repos.len();
    mgit_config.repos = Some(final_repos);
    tracing::info!(
        "{} repos are added, {} files are scanned.",
        repo_count,
        file_count
    );

    let toml_string = mgit_config.serialize();
    tokio::fs::write(config_path, toml_string)
        .await
        .map_err(|e| crate::error::MgitError::OpsError {
            message: format!("Failed to write file .gitrepos: {}", e),
        })?;

    Ok(StyleMessage::update_config_succ())
}
