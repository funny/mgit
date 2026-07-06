//! Common version-check logic shared between CLI (`mgit upgrade`) and GUI (Check for Updates).
//!
//! The default path (`latest_tag`) uses an HTTP redirect trick to avoid GitHub API rate limits.
//! Only the `--pre` path hits the API (rare, acceptable trade-off).

use std::time::Duration;

use semver::Version;
use serde::Deserialize;

use crate::error::{MgitError, MgitResult};

// --- internal API types (used by the `--pre` and specific-tag paths) ---

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

/// A single release asset exposed to callers.
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
}

/// The latest usable release found on GitHub.
#[derive(Debug)]
pub struct LatestRelease {
    pub version: Version,
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

// --- public API ---

/// Get the latest release tag without calling the REST API.
///
/// Follows the `releases/latest` web-page 302 redirect and extracts the
/// tag from the final URL.  No rate-limit concerns.
pub async fn latest_tag(repo: &str) -> MgitResult<String> {
    let client = build_client()?;
    let url = format!("https://github.com/{repo}/releases/latest");

    let resp = client
        .head(&url)
        .send()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })?;

    let final_url = resp.url().to_string();
    let tag = final_url
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();

    if tag.is_empty() || tag == "releases" {
        return Err(MgitError::UpgradeNoRelease);
    }
    Ok(tag)
}

/// Query GitHub Releases for `repo` ("owner/repo") and return the latest semver release.
///
/// * `allow_pre` — when `true`, pre-release tags (beta, rc, …) are included in the
///   candidate pool; when `false` only bare `x.y.z` releases are considered.
///
/// **Note:** this uses the GitHub REST API and is subject to rate limits.
/// Prefer `latest_tag()` unless you specifically need pre-release filtering.
pub async fn check_latest_release(repo: &str, allow_pre: bool) -> MgitResult<LatestRelease> {
    let client = build_client()?;
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=100");

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        if status == 403 {
            if let Some(remaining) = resp.headers().get("x-ratelimit-remaining") {
                if remaining == "0" {
                    return Err(MgitError::UpgradeRateLimited);
                }
            }
        }
        let body = resp.text().await.unwrap_or_default();
        return Err(MgitError::UpgradeHttpStatus { status, body });
    }

    let releases: Vec<GhRelease> = resp
        .json()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: format!("decode: {e}") })?;

    let mut candidates: Vec<(Version, GhRelease)> = releases
        .into_iter()
        .filter_map(|r| {
            let v = Version::parse(&r.tag_name).ok()?;
            if !allow_pre && (!v.pre.is_empty() || !v.build.is_empty()) {
                None
            } else {
                Some((v, r))
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates
        .into_iter()
        .next()
        .map(|(v, r)| LatestRelease {
            version: v,
            tag_name: r.tag_name,
            assets: r
                .assets
                .into_iter()
                .map(|a| ReleaseAsset {
                    name: a.name,
                    download_url: a.browser_download_url,
                })
                .collect(),
        })
        .ok_or(MgitError::UpgradeNoRelease)
}

/// Fetch a specific release by its tag name (e.g. "2.1.0" or "2.1.0-beta.1").
/// Returns `UpgradeNoRelease` if the tag does not exist (GitHub 404).
///
/// Uses the GitHub REST API — one-off request, rate-limit risk is low.
pub async fn fetch_release_by_tag(repo: &str, tag: &str) -> MgitResult<LatestRelease> {
    let client = build_client()?;
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })?;

    if resp.status().as_u16() == 404 {
        return Err(MgitError::UpgradeNoRelease);
    }
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(MgitError::UpgradeHttpStatus { status, body });
    }

    let r: GhRelease = resp
        .json()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: format!("decode: {e}") })?;

    Ok(LatestRelease {
        version: Version::parse(&r.tag_name).map_err(|e| {
            MgitError::UpgradeInvalidTag { tag: format!("{} ({e})", r.tag_name) }
        })?,
        tag_name: r.tag_name,
        assets: r
            .assets
            .into_iter()
            .map(|a| ReleaseAsset {
                name: a.name,
                download_url: a.browser_download_url,
            })
            .collect(),
    })
}

fn build_client() -> MgitResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("mgit-upgrade/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })
}
