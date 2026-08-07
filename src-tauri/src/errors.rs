use thiserror::Error;

/// Custom error types for the download application
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Process failed: {0}")]
    ProcessFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Sidecar error: {0}")]
    Sidecar(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Download cancelled by user")]
    Cancelled,

    #[error("Quality not available: {0}")]
    QualityNotAvailable(String),

    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<DownloadError> for String {
    fn from(error: DownloadError) -> Self {
        error.to_string()
    }
}

/// Determine if an error is retryable
pub fn is_retryable_error(error: &DownloadError) -> bool {
    matches!(
        error,
        DownloadError::Network(_) | DownloadError::RateLimit(_) | DownloadError::ProcessFailed(_)
    )
}

/// Determine if an error is a network error
pub fn is_network_error(stderr: &str) -> bool {
    stderr.contains("Unable to download")
        || stderr.contains("HTTP Error")
        || stderr.contains("Connection")
        || stderr.contains("timeout")
        || stderr.contains("network")
}

/// Determine if an error is a rate limit error
pub fn is_rate_limit_error(stderr: &str) -> bool {
    stderr.contains("rate limit") || stderr.contains("429") || stderr.contains("Too Many Requests")
}

/// Determine if an error is an authentication error
pub fn is_auth_error(stderr: &str) -> bool {
    stderr.contains("Sign in")
        || stderr.contains("Private video")
        || stderr.contains("members-only")
        || stderr.contains("This video is only available")
        || stderr.contains("login required")
}

/// Determine if an error is a DPAPI cookie decryption error (Windows Chrome/Edge)
pub fn is_dpapi_error(stderr: &str) -> bool {
    stderr.contains("Failed to decrypt with DPAPI")
        || stderr.contains("DPAPI")
        || (stderr.contains("decrypt") && stderr.contains("cookie"))
}

/// Determine if an error is related to ffmpeg/merge issues
pub fn is_ffmpeg_error(stderr: &str) -> bool {
    (stderr.contains("ffmpeg") || stderr.contains("Merger") || stderr.contains("merge"))
        && (stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("NoneType")
            || stderr.contains("'lower'")
            || stderr.contains("FFmpeg"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        assert_eq!(
            DownloadError::InvalidUrl("bad".into()).to_string(),
            "Invalid URL: bad"
        );
        assert_eq!(
            DownloadError::InvalidInput("bad".into()).to_string(),
            "Invalid input: bad"
        );
        assert_eq!(
            DownloadError::Network("offline".into()).to_string(),
            "Network error: offline"
        );
        assert_eq!(
            DownloadError::ProcessFailed("exit 1".into()).to_string(),
            "Process failed: exit 1"
        );
        assert_eq!(
            DownloadError::Sidecar("missing".into()).to_string(),
            "Sidecar error: missing"
        );
        assert_eq!(
            DownloadError::Authentication("login".into()).to_string(),
            "Authentication error: login"
        );
        assert_eq!(
            DownloadError::RateLimit("429".into()).to_string(),
            "Rate limit exceeded: 429"
        );
        assert_eq!(
            DownloadError::Cancelled.to_string(),
            "Download cancelled by user"
        );
        assert_eq!(
            DownloadError::QualityNotAvailable("8k".into()).to_string(),
            "Quality not available: 8k"
        );
        assert_eq!(
            DownloadError::BrowserNotFound("firefox".into()).to_string(),
            "Browser not found: firefox"
        );
        assert_eq!(
            DownloadError::ParseError("json".into()).to_string(),
            "Failed to parse output: json"
        );
        assert_eq!(
            DownloadError::Unknown("???".into()).to_string(),
            "Unknown error: ???"
        );
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let error: DownloadError = io_error.into();
        assert!(matches!(error, DownloadError::Io(_)));
        assert_eq!(error.to_string(), "IO error: no such file");
    }

    #[test]
    fn test_error_into_string() {
        let message: String = DownloadError::Cancelled.into();
        assert_eq!(message, "Download cancelled by user");
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&DownloadError::Network("x".into())));
        assert!(is_retryable_error(&DownloadError::RateLimit("x".into())));
        assert!(is_retryable_error(&DownloadError::ProcessFailed(
            "x".into()
        )));

        assert!(!is_retryable_error(&DownloadError::Cancelled));
        assert!(!is_retryable_error(&DownloadError::InvalidUrl("x".into())));
        assert!(!is_retryable_error(&DownloadError::Authentication(
            "x".into()
        )));
        assert!(!is_retryable_error(&DownloadError::Unknown("x".into())));
    }

    #[test]
    fn test_is_network_error() {
        assert!(is_network_error("ERROR: Unable to download webpage"));
        assert!(is_network_error("HTTP Error 503: Service Unavailable"));
        assert!(is_network_error("Connection reset by peer"));
        assert!(is_network_error("read timeout after 30s"));
        assert!(is_network_error("network is unreachable"));

        assert!(!is_network_error(""));
        assert!(!is_network_error("Video unavailable"));
    }

    #[test]
    fn test_is_rate_limit_error() {
        assert!(is_rate_limit_error("You have exceeded the rate limit"));
        assert!(is_rate_limit_error("HTTP Error 429"));
        assert!(is_rate_limit_error("Too Many Requests"));

        assert!(!is_rate_limit_error(""));
        assert!(!is_rate_limit_error("HTTP Error 404"));
    }

    #[test]
    fn test_is_auth_error() {
        assert!(is_auth_error("Sign in to confirm your age"));
        assert!(is_auth_error("Private video"));
        assert!(is_auth_error("This video is members-only content"));
        assert!(is_auth_error("This video is only available to subscribers"));
        assert!(is_auth_error("login required"));

        assert!(!is_auth_error(""));
        assert!(!is_auth_error("Video unavailable"));
    }

    #[test]
    fn test_is_dpapi_error() {
        assert!(is_dpapi_error("Failed to decrypt with DPAPI"));
        assert!(is_dpapi_error("DPAPI"));
        assert!(is_dpapi_error("could not decrypt cookie value"));

        assert!(!is_dpapi_error(""));
        // "decrypt" alone, without "cookie", is not a DPAPI error
        assert!(!is_dpapi_error("failed to decrypt stream"));
    }

    #[test]
    fn test_is_ffmpeg_error() {
        assert!(is_ffmpeg_error("ffmpeg not found"));
        assert!(is_ffmpeg_error("[Merger] ffmpeg does not exist"));
        assert!(is_ffmpeg_error(
            "merge failed: 'NoneType' object has no attribute"
        ));
        assert!(is_ffmpeg_error("Merger error: 'lower'"));
        assert!(is_ffmpeg_error("merge aborted: FFmpeg"));

        assert!(!is_ffmpeg_error(""));
        // Mentions ffmpeg but no failure indicator
        assert!(!is_ffmpeg_error("[ffmpeg] Destination: video.mp4"));
        // Failure indicator but unrelated to ffmpeg
        assert!(!is_ffmpeg_error("cookie file not found"));
    }
}
