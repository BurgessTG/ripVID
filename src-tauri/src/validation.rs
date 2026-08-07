// Security validation module for ripVID
// Provides comprehensive input validation to prevent injection attacks

use std::path::{Path, PathBuf};
use url::Url;

/// Validates a URL to prevent command injection and ensure safe URL schemes
///
/// # Security Checks:
/// - Only allows http:// and https:// schemes
/// - Rejects URLs with dangerous shell characters
/// - Validates URL structure
/// - Prevents protocol smuggling
///
/// # Arguments
/// * `url_str` - The URL string to validate
///
/// # Returns
/// * `Ok(String)` - Validated URL if safe
/// * `Err(String)` - Error message if validation fails
pub fn validate_url(url_str: &str) -> Result<String, String> {
    // Check for empty or whitespace-only URLs
    if url_str.trim().is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // Check URL length to prevent DoS
    if url_str.len() > 2048 {
        return Err("URL is too long (max 2048 characters)".to_string());
    }

    // Parse the URL to validate structure
    let parsed_url = Url::parse(url_str).map_err(|e| format!("Invalid URL format: {}", e))?;

    // Only allow http and https schemes
    let scheme = parsed_url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "Unsupported URL scheme '{}'. Only http and https are allowed",
            scheme
        ));
    }

    // Validate that the URL has a host
    if parsed_url.host_str().is_none() {
        return Err("URL must have a valid host".to_string());
    }

    // Check for dangerous characters that could be used in shell injection
    // Even though we use proper argument passing, defense in depth
    let dangerous_chars = &[
        '<', '>', '|', '&', ';', '`', '$', '(', ')', '{', '}', '[', ']', '!', '#',
    ];

    // Only check dangerous characters in certain parts of the URL
    // Allow them in query parameters as they may be URL-encoded
    let url_without_query = if let Some(idx) = url_str.find('?') {
        &url_str[..idx]
    } else {
        url_str
    };

    for &ch in dangerous_chars {
        if url_without_query.contains(ch) {
            return Err(format!(
                "URL contains dangerous character '{}' which is not allowed",
                ch
            ));
        }
    }

    // Additional check: ensure no null bytes
    if url_str.contains('\0') {
        return Err("URL contains null bytes".to_string());
    }

    // Additional check: ensure no control characters
    for ch in url_str.chars() {
        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            return Err(format!("URL contains invalid control character: {:?}", ch));
        }
    }

    // Return the validated URL
    Ok(parsed_url.to_string())
}

/// Validates a file path to prevent path traversal attacks
///
/// # Security Checks:
/// - Prevents .. (parent directory) traversal
/// - Ensures path is absolute
/// - Validates path exists or parent exists
/// - Restricts to user's home directory or downloads
/// - Normalizes path using canonicalize when possible
///
/// # Arguments
/// * `path_str` - The file path to validate
/// * `allow_nonexistent` - Whether to allow paths that don't exist yet
///
/// # Returns
/// * `Ok(PathBuf)` - Validated and normalized path if safe
/// * `Err(String)` - Error message if validation fails
pub fn validate_path(path_str: &str, allow_nonexistent: bool) -> Result<PathBuf, String> {
    // Check for empty paths
    if path_str.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    // Check path length
    if path_str.len() > 4096 {
        return Err("Path is too long (max 4096 characters)".to_string());
    }

    let path = Path::new(path_str);

    // Check for null bytes
    if path_str.contains('\0') {
        return Err("Path contains null bytes".to_string());
    }

    // Reject paths with .. components (path traversal)
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(
                "Path contains '..' which is not allowed (path traversal attempt)".to_string(),
            );
        }
    }

    // Get the absolute path
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        return Err("Path must be absolute".to_string());
    };

    // If path exists, canonicalize it (resolves symlinks and normalizes)
    let normalized_path = if absolute_path.exists() {
        absolute_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?
    } else if allow_nonexistent {
        // For non-existent paths, validate the parent directory
        if let Some(parent) = absolute_path.parent() {
            if !parent.exists() {
                return Err(format!(
                    "Parent directory does not exist: {}",
                    parent.display()
                ));
            }
            // Canonicalize parent and append filename
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize parent directory: {}", e))?;

            if let Some(filename) = absolute_path.file_name() {
                canonical_parent.join(filename)
            } else {
                return Err("Invalid path: no filename".to_string());
            }
        } else {
            return Err("Path has no parent directory".to_string());
        }
    } else {
        return Err(format!("Path does not exist: {}", absolute_path.display()));
    };

    // Check that the normalized path is within allowed directories
    if let Some(home_dir) = dirs::home_dir() {
        // Check if path is under home directory
        if !normalized_path.starts_with(&home_dir) {
            // Also allow system temp directory
            if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
                if !normalized_path.starts_with(&temp_dir) {
                    return Err(format!(
                        "Path is outside allowed directories (home or temp): {}",
                        normalized_path.display()
                    ));
                }
            } else {
                return Err(format!(
                    "Path is outside home directory: {}",
                    normalized_path.display()
                ));
            }
        }
    } else {
        // If we can't determine home directory, be more restrictive
        // Only allow if explicitly in a safe location
        let path_str = normalized_path.to_string_lossy().to_lowercase();
        let safe_prefixes = if cfg!(windows) {
            vec!["c:\\users\\", "c:\\temp\\", "c:\\tmp\\"]
        } else {
            vec!["/home/", "/tmp/", "/var/tmp/"]
        };

        if !safe_prefixes
            .iter()
            .any(|prefix| path_str.starts_with(prefix))
        {
            return Err(
                "Cannot validate path: home directory unknown and path not in safe location"
                    .to_string(),
            );
        }
    }

    // Additional check: ensure path doesn't contain suspicious patterns
    let path_str_lower = normalized_path.to_string_lossy().to_lowercase();

    // Block access to sensitive system directories
    let blocked_paths = if cfg!(windows) {
        vec![
            "\\windows\\system32\\",
            "\\windows\\syswow64\\",
            "\\program files\\",
            "\\programdata\\",
        ]
    } else {
        vec!["/etc/", "/boot/", "/sys/", "/proc/", "/root/"]
    };

    for blocked in blocked_paths {
        if path_str_lower.contains(blocked) {
            return Err(format!(
                "Access to system directory is not allowed: {}",
                blocked
            ));
        }
    }

    Ok(normalized_path)
}

/// Validates an output path for downloads
/// More permissive than validate_path as it needs to allow non-existent files
///
/// # Arguments
/// * `path_str` - The output file path
///
/// # Returns
/// * `Ok(PathBuf)` - Validated path if safe
/// * `Err(String)` - Error message if validation fails
pub fn validate_output_path(path_str: &str) -> Result<PathBuf, String> {
    validate_path(path_str, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A directory under the user's home directory that is removed on drop.
    /// `validate_path` only accepts paths inside the home or temp directory.
    struct TempHomeDir(PathBuf);

    impl TempHomeDir {
        fn new() -> Self {
            let unique = format!(
                "ripvid-validation-test-{}-{}",
                std::process::id(),
                TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
            );
            let path = dirs::home_dir()
                .expect("home directory is required for these tests")
                .join(unique);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn join(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().to_string()
        }

        fn path(&self) -> String {
            self.0.to_string_lossy().to_string()
        }
    }

    impl Drop for TempHomeDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://www.youtube.com/watch?v=test").is_ok());
        assert!(validate_url("http://example.com/video").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_dangerous_chars() {
        assert!(validate_url("https://example.com/video;rm -rf /").is_err());
        assert!(validate_url("https://example.com/`whoami`").is_err());
        assert!(validate_url("https://example.com/$(command)").is_err());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(validate_url("").is_err());
        assert!(validate_url("   ").is_err());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../../../etc/passwd", false).is_err());
        assert!(validate_path("/home/user/../../etc/passwd", false).is_err());
    }

    #[test]
    fn test_validate_path_null_bytes() {
        assert!(validate_path("/home/user/file\0.txt", false).is_err());
    }

    #[test]
    fn test_validate_url_normalizes_and_returns_parsed_url() {
        assert_eq!(
            validate_url("https://example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn test_validate_url_too_long() {
        let long_url = format!("https://example.com/{}", "a".repeat(2048));
        let error = validate_url(&long_url).unwrap_err();
        assert!(error.contains("too long"));
    }

    #[test]
    fn test_validate_url_malformed() {
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("https://").is_err());
    }

    #[test]
    fn test_validate_url_allows_dangerous_characters_in_query_only() {
        // Query strings may legitimately contain these characters
        assert!(validate_url("https://example.com/watch?v=a&list=b").is_ok());
        // The same character before the query is rejected
        assert!(validate_url("https://example.com/a&b").is_err());
    }

    #[test]
    fn test_validate_url_rejects_control_characters() {
        let error = validate_url("https://example.com/vid\u{7}eo").unwrap_err();
        assert!(error.contains("control character"));
    }

    #[test]
    fn test_validate_path_accepts_existing_directory_in_home() {
        let dir = TempHomeDir::new();
        let validated = validate_path(&dir.path(), false).unwrap();
        assert!(validated.is_absolute());
        assert!(validated.exists());
    }

    #[test]
    fn test_validate_path_rejects_missing_path_when_not_allowed() {
        let dir = TempHomeDir::new();
        let error = validate_path(&dir.join("missing.mp4"), false).unwrap_err();
        assert!(error.contains("does not exist"));
    }

    #[test]
    fn test_validate_path_allows_missing_file_with_existing_parent() {
        let dir = TempHomeDir::new();
        let target = dir.join("video.mp4");
        let validated = validate_path(&target, true).unwrap();
        assert_eq!(validated.file_name().unwrap(), "video.mp4");
    }

    #[test]
    fn test_validate_path_rejects_missing_parent() {
        let dir = TempHomeDir::new();
        let target = dir.join("nested/video.mp4");
        let error = validate_path(&target, true).unwrap_err();
        assert!(error.contains("Parent directory does not exist"));
    }

    #[test]
    fn test_validate_path_rejects_relative_and_empty_paths() {
        assert!(validate_path("relative/video.mp4", true).is_err());
        assert_eq!(validate_path("", true).unwrap_err(), "Path cannot be empty");
        assert!(validate_path("   ", true).is_err());
    }

    #[test]
    fn test_validate_path_too_long() {
        let long_path = format!("/{}", "a".repeat(4096));
        let error = validate_path(&long_path, true).unwrap_err();
        assert!(error.contains("too long"));
    }

    #[test]
    fn test_validate_path_rejects_outside_home_and_temp() {
        let outside = if cfg!(windows) {
            "C:\\Windows\\System32"
        } else {
            "/usr/local"
        };
        assert!(validate_path(outside, false).is_err());
    }

    #[test]
    fn test_validate_path_rejects_system_directory_names() {
        let dir = TempHomeDir::new();
        let subdir = if cfg!(windows) {
            "program files"
        } else {
            "etc"
        };
        std::fs::create_dir(dir.join(subdir)).unwrap();

        let blocked = format!("{}/video.mp4", dir.join(subdir));
        let error = validate_path(&blocked, true).unwrap_err();
        assert!(error.contains("system directory"));
    }

    #[test]
    fn test_validate_output_path_allows_nonexistent_file() {
        let dir = TempHomeDir::new();
        let target = dir.join("output.mp3");
        assert!(validate_output_path(&target).is_ok());
        assert!(validate_output_path(&dir.join("nested/output.mp3")).is_err());
    }
}
