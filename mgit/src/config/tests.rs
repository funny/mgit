//! Unit tests for config module

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::config::RepoConfig;

    /// Test RepoId::new
    #[test]
    fn test_repo_id_new() {
        let id = crate::config::RepoId::new(1, "foo/bar");
        assert_eq!(id.id, 1);
        assert_eq!(id.repo, "foo_bar");
    }

    /// Test RepoId::new with path separators
    #[test]
    fn test_repo_id_new_with_separators() {
        let id = crate::config::RepoId::new(2, "path/to/repo");
        assert_eq!(id.id, 2);
        assert_eq!(id.repo, "path_to_repo");
    }

    /// Test RepoConfig creation
    #[test]
    fn test_repo_config_creation() {
        let config = RepoConfig {
            local: None,
            remote: None,
            branch: None,
            tag: None,
            commit: None,
            sparse: None,
            labels: None,
        };
        assert!(config.local.is_none());
        assert!(config.remote.is_none());
        assert!(config.branch.is_none());
        assert!(config.tag.is_none());
        assert!(config.commit.is_none());
        assert!(config.sparse.is_none());
        assert!(config.labels.is_none());
    }

    /// Test RepoConfig clone
    #[test]
    fn test_repo_config_clone() {
        let config = RepoConfig {
            local: Some("test".to_string()),
            remote: Some("https://example.com/repo.git".to_string()),
            branch: Some("main".to_string()),
            tag: None,
            commit: None,
            sparse: None,
            labels: None,
        };
        let cloned = config.clone();
        assert_eq!(config.local, cloned.local);
        assert_eq!(config.remote, cloned.remote);
        assert_eq!(config.branch, cloned.branch);
    }

    /// Test repos_to_map_with_ignore with empty repos
    #[test]
    fn test_repos_to_map_empty() {
        let repos = Vec::new();
        let result = crate::config::repos_to_map_with_ignore(repos, None, None);
        assert!(result.is_empty());
    }

    /// Test repos_to_map_with_ignore with single repo
    #[test]
    fn test_repos_to_map_single() {
        let repos = vec![RepoConfig {
            local: Some("test".to_string()),
            remote: Some("https://example.com/repo.git".to_string()),
            branch: Some("main".to_string()),
            tag: None,
            commit: None,
            sparse: None,
            labels: None,
        }];
        let result = crate::config::repos_to_map_with_ignore(repos, None, None);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&0));
    }

    /// Test repos_to_map_with_ignore with ignore filter
    #[test]
    fn test_repos_to_map_with_ignore_filter() {
        let repos = vec![
            RepoConfig {
                local: Some("repo1".to_string()),
                remote: Some("https://example.com/repo1.git".to_string()),
                branch: None,
                tag: None,
                commit: None,
                sparse: None,
                labels: None,
            },
            RepoConfig {
                local: Some("repo2".to_string()),
                remote: Some("https://example.com/repo2.git".to_string()),
                branch: None,
                tag: None,
                commit: None,
                sparse: None,
                labels: None,
            },
        ];
        let ignore = Some(vec!["repo1".to_string()]);
        let result = crate::config::repos_to_map_with_ignore(repos, ignore.as_ref(), None);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&1));
    }

    /// Test repos_to_map_with_ignore has correct count
    #[test]
    fn test_repos_to_map_has_correct_count() {
        let repos = vec![
            RepoConfig {
                local: Some("a".to_string()),
                remote: None,
                branch: None,
                tag: None,
                commit: None,
                sparse: None,
                labels: None,
            },
            RepoConfig {
                local: Some("b".to_string()),
                remote: None,
                branch: None,
                tag: None,
                commit: None,
                sparse: None,
                labels: None,
            },
            RepoConfig {
                local: Some("c".to_string()),
                remote: None,
                branch: None,
                tag: None,
                commit: None,
                sparse: None,
                labels: None,
            },
        ];
        let result: HashMap<usize, RepoConfig> =
            crate::config::repos_to_map_with_ignore(repos, None, None);
        // HashMap doesn't guarantee order, but we should have 3 entries
        assert_eq!(result.len(), 3);
        // All keys should be unique and within range
        let keys: HashSet<_> = result.keys().collect();
        assert_eq!(keys.len(), 3);
    }
}
