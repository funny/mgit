use std::collections::BTreeSet;

use crate::config::RepoConfig;

pub fn filter<'a>(
    repos: &'a [RepoConfig],
    labels: &'a [String],
) -> impl Iterator<Item = &'a RepoConfig> {
    repos.iter().filter(move |repo| check(repo, labels))
}

pub fn check(repo: &RepoConfig, labels: &[String]) -> bool {
    let Some(repo_labels) = &repo.labels else {
        return true;
    };

    if labels == ["none"] {
        return false; // 暂时不考虑仓库 label配置为 none的情况
    }

    for label in labels {
        if repo_labels.contains(label) {
            return true;
        }
    }
    false
}

pub fn collect(repos: &[RepoConfig]) -> BTreeSet<&str> {
    repos
        .iter()
        .filter_map(|x| x.labels.as_deref())
        .flatten()
        .map(|x| x.as_str())
        .collect()
}
