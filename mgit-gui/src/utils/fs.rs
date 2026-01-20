use std::path::{Path, PathBuf};
use std::{fs, io};

pub(crate) fn atomic_write_file(path: &Path, content: &[u8]) -> io::Result<()> {
    let unique = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_nanos(),
        Err(_) => 0,
    };
    atomic_write_file_with_unique(path, content, unique)
}

fn atomic_write_file_with_unique(path: &Path, content: &[u8], unique: u128) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let tmp_path = parent.join(format!("{}.tmp-{}", file_name, unique));
    let bak_path = parent.join(format!("{}.bak-{}", file_name, unique));

    {
        let mut f = fs::File::create(&tmp_path)?;
        use io::Write;
        f.write_all(content)?;
        f.sync_all()?;
    }

    let mut backup_path: Option<PathBuf> = None;
    if path.exists() {
        if bak_path.exists() {
            let _ = fs::remove_file(&bak_path);
        }
        if let Err(e) = fs::rename(path, &bak_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }
        backup_path = Some(bak_path);
    }

    match fs::rename(&tmp_path, path) {
        Ok(()) => {
            if let Some(bak_path) = backup_path {
                let _ = fs::remove_file(bak_path);
            }
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            if let Some(bak_path) = backup_path {
                let _ = fs::rename(&bak_path, path);
                let _ = fs::remove_file(bak_path);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_dir(prefix: &str) -> PathBuf {
        let unique = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!("{}-{}", prefix, unique))
    }

    #[test]
    fn atomic_write_creates_file() {
        let dir = unique_dir("mgit-gui-atomic-write");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".gitrepos");

        atomic_write_file(&path, b"hello").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"hello");
    }

    #[test]
    fn atomic_write_overwrites_file() {
        let dir = unique_dir("mgit-gui-atomic-write");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".gitrepos");

        atomic_write_file(&path, b"old").unwrap();
        atomic_write_file(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
    }

    #[test]
    fn atomic_write_cleans_tmp_when_backup_rename_fails() {
        let dir = unique_dir("mgit-gui-atomic-write");
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join(".gitrepos");
        fs::write(&path, b"old").unwrap();

        let unique = 1u128;
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let bak_path = dir.join(format!("{}.bak-{}", file_name, unique));
        let tmp_path = dir.join(format!("{}.tmp-{}", file_name, unique));

        fs::create_dir_all(&bak_path).unwrap();

        let err = atomic_write_file_with_unique(&path, b"new", unique).unwrap_err();
        let _ = err;

        assert!(path.is_file());
        assert!(!tmp_path.exists());
    }
}
