/// normalize path if needed
pub fn norm_path(path: impl AsRef<str>) -> String {
    let mut path = path.as_ref().replace("\\", "/");
    while path.ends_with("/") {
        path.pop();
    }
    path
}

/// if path is empty, represent it by "."
pub fn display_path(path: impl AsRef<str>) -> String {
    match path.as_ref().is_empty() {
        true => String::from("."),
        false => path.as_ref().to_string(),
    }
}
