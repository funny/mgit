use globset::GlobBuilder;

use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::MgitConfig;
use crate::error::MgitResult;
use crate::utils::label;
use crate::utils::style_message::StyleMessage;

pub struct CleanOptions {
    pub path: PathBuf,
    pub config_path: PathBuf,
    pub labels: Option<Vec<String>>,
}

impl CleanOptions {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        config_path: Option<impl AsRef<Path>>,
        labels: Option<Vec<String>>,
    ) -> Self {
        let path = path.map_or(env::current_dir().unwrap(), |p| p.as_ref().to_path_buf());
        let config_path = config_path.map_or(path.join(".gitrepos"), |p| p.as_ref().to_path_buf());
        Self {
            path,
            config_path,
            labels,
        }
    }
}

pub async fn clean_repo(options: CleanOptions) -> MgitResult<StyleMessage> {
    let path = &options.path;
    let config_path = &options.config_path;

    tracing::info!("Clean Status:");

    // if directory doesn't exist, finsh clean
    if !path.is_dir() {
        return Err(crate::error::MgitError::DirNotFound { path: path.clone() });
    }
    // check if .gitrepos exists
    if !config_path.is_file() {
        return Err(crate::error::MgitError::ConfigFileNotFound {
            path: config_path.clone(),
        });
    }
    // load config file(like .gitrepos)
    let mgit_config =
        MgitConfig::load(config_path).ok_or(crate::error::MgitError::LoadConfigFailed {
            source: std::io::Error::new(std::io::ErrorKind::Other, "Failed to load config"),
        })?;

    let mut repo_configs = if let Some(repos) = mgit_config.repos {
        repos
    } else {
        return Ok(StyleMessage::new().plain_text("No repos to clean"));
    };

    if let Some(labels) = options.labels {
        repo_configs = label::filter(&repo_configs, &labels).cloned().collect();
    }

    let config_repo_paths: Vec<PathBuf> = repo_configs
        .iter()
        .map(|item| item.local.as_ref().unwrap())
        .map(PathBuf::from)
        .collect();

    // search for git repos and create .gitrepos file
    let glob = GlobBuilder::new("**/.git")
        .literal_separator(true)
        .build()
        .unwrap()
        .compile_matcher();

    let input_path = path.to_owned();

    // WalkDir is blocking, so we wrap it in spawn_blocking
    let input_path_clone = input_path.clone();
    let config_repo_paths_clone = config_repo_paths.clone();

    let mut unused_paths = tokio::task::spawn_blocking(move || {
        let mut unused = Vec::new();
        let mut it = WalkDir::new(&input_path_clone).into_iter();

        loop {
            let entry = match it.next() {
                None => break,
                Some(Err(err)) => panic!("ERROR: {}", err),
                Some(Ok(entry)) => entry,
            };
            let path = entry.path();

            if glob.is_match(path) {
                // get relative path
                let mut pb = path.to_path_buf();
                pb.pop();
                let rel_path = pb.strip_prefix(&input_path_clone).unwrap().to_path_buf();

                if !config_repo_paths_clone.contains(&rel_path) {
                    unused.push(rel_path);
                }

                // just skip go into .git/ folder and continue
                it.skip_current_dir();
                continue;
            }
        }
        unused
    })
    .await
    .map_err(|e| crate::error::MgitError::OpsError {
        message: format!("Failed to walk directory: {}", e),
    })?;

    unused_paths.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    // remvoe unused repositories
    let mut count: u32 = 0;
    for unused_path in unused_paths {
        // find contianed repo path
        let contained_paths = find_contained_paths(&unused_path, &config_repo_paths);

        // remove unused directory
        if !contained_paths.is_empty() {
            if let Err(e) = remove_unused_files(&input_path, &unused_path, &contained_paths).await {
                tracing::error!(message = %StyleMessage::remove_file_failed(&unused_path, &e));
            };
        } else {
            match tokio::fs::remove_dir_all(input_path.join(&unused_path)).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(crate::error::MgitError::OpsError {
                        message: format!("Failed to remove dir {:?}: {}", unused_path, e),
                    });
                }
            }
        }
        count += 1;

        tracing::info!(message = %StyleMessage::remove_file_succ(&unused_path));
    }

    // show statistics info
    Ok(StyleMessage::remove_repo_succ(count))
}

fn find_contained_paths(unused_path: &Path, config_repo_paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut contained_paths: Vec<PathBuf> = Vec::new();

    for config_repo_path in config_repo_paths {
        // add contained paths
        if config_repo_path.as_path().starts_with(unused_path) {
            contained_paths.push(config_repo_path.to_path_buf());
        }
    }

    contained_paths
}

async fn remove_unused_files(
    base_path: impl AsRef<Path>,
    unused_path: impl AsRef<Path>,
    contained_paths: &Vec<PathBuf>,
) -> MgitResult<()> {
    let full_path = base_path.as_ref().join(&unused_path);
    let base_path_buf = base_path.as_ref().to_path_buf();
    let contained_paths = contained_paths.clone();

    // WalkDir is blocking
    tokio::task::spawn_blocking(move || {
        // forearch files/folders begin with unused path
        let mut it = WalkDir::new(&full_path).into_iter();
        loop {
            let entry = match it.next() {
                None => break,
                Some(Err(err)) => panic!("ERROR: {}", err),
                Some(Ok(entry)) => entry,
            };

            // get file/folder path
            let file_path = entry.path();
            let rel_path = file_path
                .strip_prefix(&base_path_buf)
                .unwrap()
                .to_path_buf();

            // if the path is contained path, skip the path
            if contained_paths.contains(&rel_path) {
                it.skip_current_dir();
            }
            // if the path is not the parent of contained path, continue
            else if file_path.is_dir()
                && find_contained_paths(&rel_path, &contained_paths).is_empty()
            {
                std::fs::remove_dir_all(file_path).unwrap();
                it.skip_current_dir();
            }
            // otherwise, delete the file/folder
            else if file_path.is_file() {
                std::fs::remove_file(file_path).unwrap();
            }
        }
    })
    .await
    .map_err(|e| crate::error::MgitError::OpsError {
        message: format!("Failed to remove unused files: {}", e),
    })?;

    Ok(())
}
