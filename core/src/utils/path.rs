use std::ops::Add;
use std::path::Path;

pub trait PathExtension {
    /// normalize path if needed
    fn norm_path(&self) -> String;

    /// if path is empty, represent it by "."
    fn display_path(&self) -> String;
}

impl<T: AsRef<Path>> PathExtension for T {
    fn norm_path(&self) -> String {
        let mut path = self.as_ref().display().to_string().replace('\\', "/");
        path = path.trim_end_matches('/').to_string();
        path
    }

    fn display_path(&self) -> String {
        let mut path = self.as_ref().display().to_string();
        if path.is_empty() {
            path = path.add(".");
        }
        path
    }
}
