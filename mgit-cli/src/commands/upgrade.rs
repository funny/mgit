use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use clap::Args;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;

use mgit::error::{MgitError, MgitResult};
use mgit::utils::upgrade_check::{self, ReleaseAsset};

use crate::commands::CliCommand;

/// The "owner/repo" used for fetching release metadata.
/// Separate from CARGO_PKG_REPOSITORY so forks can point to their own release feed.
const UPGRADE_REPO: &str = "yhx0516/mgit";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Args)]
/// Upgrade mgit CLI to the latest release
pub(crate) struct UpgradeCommand {
    /// Force re-installation even if already on the latest version
    #[arg(long)]
    pub force: bool,

    /// Include pre-release versions (beta, rc, etc.)
    #[arg(long)]
    pub pre: bool,
}

impl CliCommand for UpgradeCommand {
    async fn exec(self) -> MgitResult<()> {
        let current_ver = Version::parse(CURRENT_VERSION)
            .map_err(|e| MgitError::UpgradeInvalidTag { tag: format!("{CURRENT_VERSION} ({e})") })?;
        println!("current version: {current_ver}");

        let target = pick_target()?;

        println!("fetching latest release from github.com/{} ...", UPGRADE_REPO);
        let latest = upgrade_check::check_latest_release(UPGRADE_REPO, self.pre).await?;
        println!("latest version:  {}", latest.version);

        if !self.force && latest.version <= current_ver {
            println!("already up to date.");
            return Ok(());
        }

        let asset = pick_asset(&latest.assets, &latest.version, target)?;
        println!("downloading {} ...", asset.name);

        let client = build_client()?;
        let bytes = download_with_progress(&client, &asset.download_url).await?;
        let binary = extract_binary(&bytes)?;

        let exe = std::env::current_exe()
            .map_err(|e| MgitError::UpgradeSelfReplaceFailed { message: format!("current_exe: {e}") })?;
        println!("replacing {} ...", exe.display());
        replace_self(&exe, &binary)?;

        println!("upgraded to {}.", latest.version);
        Ok(())
    }
}

fn pick_target() -> MgitResult<&'static str> {
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    match (os, arch) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-musl"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        _ => Err(MgitError::UpgradeUnsupportedPlatform {
            os: os.to_string(),
            arch: arch.to_string(),
        }),
    }
}

fn build_client() -> MgitResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("mgit-upgrade/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })
}

fn pick_asset<'a>(assets: &'a [ReleaseAsset], version: &Version, target: &str) -> MgitResult<&'a ReleaseAsset> {
    let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
    let expected = format!("mgit-cli-{version}-{target}.{ext}");
    assets
        .iter()
        .find(|a| a.name == expected)
        .ok_or_else(|| MgitError::UpgradeAssetNotFound { target: target.to_string() })
}

async fn download_with_progress(client: &reqwest::Client, url: &str) -> MgitResult<Vec<u8>> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(MgitError::UpgradeHttpStatus { status, body });
    }

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:30.cyan/white} {bytes}/{total_bytes} ({bytes_per_sec})")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.enable_steady_tick(Duration::from_millis(500));

    let mut stream = resp.bytes_stream();
    let mut buf = Vec::with_capacity(total as usize);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|e| MgitError::UpgradeNetworkError { message: e.to_string() })?;
        pb.inc(chunk.len() as u64);
        buf.extend_from_slice(&chunk);
    }
    pb.finish_and_clear();

    Ok(buf)
}

fn extract_binary(bytes: &[u8]) -> MgitResult<Vec<u8>> {
    if cfg!(target_os = "windows") {
        extract_zip(bytes)
    } else {
        extract_tar_gz(bytes)
    }
}

fn extract_tar_gz(bytes: &[u8]) -> MgitResult<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let decoder = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(decoder);
    for entry in archive
        .entries()
        .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?
    {
        let mut entry = entry
            .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
        let path = entry
            .path()
            .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
        if path.file_name() == Some(std::ffi::OsStr::new("mgit")) {
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
            return Ok(data);
        }
    }
    Err(MgitError::UpgradeBinaryNotFound { name: "mgit".to_string() })
}

fn extract_zip(bytes: &[u8]) -> MgitResult<Vec<u8>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
        let name = file.name().to_string();
        if name.ends_with("mgit.exe") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| MgitError::UpgradeArchiveFailed { message: e.to_string() })?;
            return Ok(data);
        }
    }
    Err(MgitError::UpgradeBinaryNotFound { name: "mgit.exe".to_string() })
}

fn replace_self(current_exe: &Path, binary: &[u8]) -> MgitResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::NamedTempFile;

        let dir = current_exe
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let mut tmp = NamedTempFile::new_in(dir).map_err(|e| {
            MgitError::UpgradeSelfReplaceFailed { message: format!("temp create: {e}") }
        })?;
        tmp.write_all(binary).map_err(|e| {
            MgitError::UpgradeSelfReplaceFailed { message: format!("temp write: {e}") }
        })?;
        fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o755)).map_err(|e| {
            MgitError::UpgradeSelfReplaceFailed { message: format!("chmod: {e}") }
        })?;
        // `fs::rename` is atomic on Unix; the running binary's inode stays valid.
        // NamedTempFile's drop will no-op (file moved away) on success, or clean up on failure.
        fs::rename(tmp.path(), current_exe).map_err(|e| {
            MgitError::UpgradeSelfReplaceFailed { message: format!("rename: {e}") }
        })?;
        // tmp still guards the temp path; drop after rename is a no-op or silent ENOENT.
        std::mem::forget(tmp);
        Ok(())
    }

    #[cfg(windows)]
    {
        // Windows allows renaming a running .exe but not overwriting it.
        // Rename current → .old, write new, rollback on write failure.
        let old = current_exe.with_extension("old");
        let _ = fs::remove_file(&old);
        fs::rename(current_exe, &old).map_err(|e| {
            MgitError::UpgradeSelfReplaceFailed { message: format!("rename to .old: {e}") }
        })?;
        if let Err(e) = fs::write(current_exe, binary) {
            let _ = fs::rename(&old, current_exe);
            return Err(MgitError::UpgradeSelfReplaceFailed {
                message: format!("write: {e}"),
            }
            .into());
        }
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = binary;
        Err(MgitError::UpgradeSelfReplaceFailed {
            message: format!("unsupported platform for self-replace: {}", current_exe.display()),
        }
        .into())
    }
}
