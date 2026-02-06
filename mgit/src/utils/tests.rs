//! Unit tests for utils module

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::utils::{current_dir, current_dir_or, get_current_dir};

    /// Test current_dir helper functions
    #[test]
    fn test_current_dir_returns_valid_path() {
        let path = current_dir();
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn test_current_dir_or_with_fallback() {
        let fallback = Path::new("/nonexistent");
        let path = current_dir_or(fallback);
        // Should return current dir since it exists
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn test_get_current_dir_returns_result() {
        let result = get_current_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.is_dir());
    }
}

#[cfg(test)]
mod path_extension_tests {
    use std::path::Path;

    use crate::utils::path::PathExtension;

    /// Test PathExtension trait
    #[test]
    fn test_norm_path_basic() {
        let path = Path::new("foo/bar");
        let norm = path.norm_path();
        // norm_path should handle basic paths
        assert!(!norm.is_empty());
    }

    #[test]
    fn test_display_path() {
        let path = Path::new("foo/bar/baz");
        let display = path.display_path();
        // display_path should return a displayable string
        assert!(!display.is_empty());
    }
}

#[cfg(test)]
mod style_message_tests {
    use crate::utils::StyleMessage;

    #[test]
    fn test_style_message_default() {
        let msg = StyleMessage::default();
        assert!(msg.to_plain_text().is_empty());
    }

    #[test]
    fn test_style_message_new() {
        let msg = StyleMessage::new();
        assert!(msg.to_plain_text().is_empty());
    }

    #[test]
    fn test_style_message_clone() {
        let msg = StyleMessage::new();
        let cloned = msg.clone();
        assert_eq!(msg.to_plain_text(), cloned.to_plain_text());
    }
}
