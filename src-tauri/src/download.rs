use crate::binary_manager::BinaryManager;
use crate::errors::{
    is_auth_error, is_dpapi_error, is_ffmpeg_error, is_network_error, is_rate_limit_error,
    DownloadError,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Strip Windows extended-length path prefix (\\?\) for yt-dlp compatibility
/// yt-dlp doesn't recognize the \\?\ prefix and treats such paths as invalid
#[cfg(target_os = "windows")]
fn strip_extended_path_prefix(path: &std::path::Path) -> String {
    const VERBATIM_PREFIX: &str = r"\\?\";
    let path_str = path.display().to_string();

    if path_str.starts_with(VERBATIM_PREFIX) {
        // Strip \\?\ prefix and convert to forward slashes (yt-dlp prefers these)
        path_str[VERBATIM_PREFIX.len()..].replace("\\", "/")
    } else {
        // Already clean, just convert backslashes to forward slashes
        path_str.replace("\\", "/")
    }
}

#[cfg(not(target_os = "windows"))]
fn strip_extended_path_prefix(path: &std::path::Path) -> String {
    path.display().to_string()
}

/// Type of download to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadType {
    Video { quality: String },
    Audio,
}

/// Progress information for downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
}

/// Handle to an active download process
pub struct DownloadHandle {
    pub id: String,
    pub child: CommandChild,
    pub url: String,
    pub output_path: String,
}

/// Configuration for browser cookie support
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub use_cookies: bool,
    pub browser: Option<String>,
}

impl BrowserConfig {
    pub fn new(use_cookies: bool) -> Self {
        Self {
            use_cookies,
            browser: if use_cookies { detect_browser() } else { None },
        }
    }
}

/// Detect which browser to use for cookies
pub fn detect_browser() -> Option<String> {
    info!("Starting browser detection for cookie extraction...");

    // Try to detect installed browsers in order of preference
    // Firefox first - doesn't have Windows DPAPI cookie encryption issues
    let browsers = vec!["firefox", "chrome", "edge"];

    for browser in browsers {
        debug!("Checking for browser: {}", browser);
        if is_browser_installed(browser) {
            info!("✓ Detected browser for cookies: {}", browser);
            return Some(browser.to_string());
        } else {
            debug!("✗ Browser not found: {}", browser);
        }
    }

    warn!("No supported browser found for cookie extraction");
    warn!("Checked: Firefox, Chrome, Edge");
    warn!("Recommendation: Install Firefox for best compatibility on Windows");
    None
}

/// Check if a browser is installed (improved detection)
fn is_browser_installed(browser: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let found = match browser {
            "firefox" => {
                // Check multiple Firefox locations
                let paths = vec![
                    "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
                    "C:\\Program Files (x86)\\Mozilla Firefox\\firefox.exe",
                ];

                for path in &paths {
                    debug!("  Checking path: {}", path);
                    if std::path::Path::new(path).exists() {
                        debug!("  ✓ Found at: {}", path);
                        return true;
                    }
                }

                // Try AppData location
                if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
                    let local_path = format!("{}\\Mozilla Firefox\\firefox.exe", appdata);
                    debug!("  Checking AppData: {}", local_path);
                    if std::path::Path::new(&local_path).exists() {
                        debug!("  ✓ Found at: {}", local_path);
                        return true;
                    }
                }

                false
            }
            "chrome" => {
                // Check multiple Chrome locations
                let paths = vec![
                    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
                ];

                for path in &paths {
                    debug!("  Checking path: {}", path);
                    if std::path::Path::new(path).exists() {
                        debug!("  ✓ Found at: {}", path);
                        return true;
                    }
                }

                // Try AppData location
                if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
                    let local_path =
                        format!("{}\\Google\\Chrome\\Application\\chrome.exe", appdata);
                    debug!("  Checking AppData: {}", local_path);
                    if std::path::Path::new(&local_path).exists() {
                        debug!("  ✓ Found at: {}", local_path);
                        return true;
                    }
                }

                false
            }
            "edge" => {
                // Check multiple Edge locations
                let paths = vec![
                    "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
                    "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                ];

                for path in &paths {
                    debug!("  Checking path: {}", path);
                    if std::path::Path::new(path).exists() {
                        debug!("  ✓ Found at: {}", path);
                        return true;
                    }
                }

                // Try where command as fallback
                debug!("  Trying 'where msedge.exe' command...");
                let where_result = Command::new("where")
                    .arg("msedge.exe")
                    .output()
                    .map(|output| {
                        let found = output.status.success();
                        if found {
                            let path = String::from_utf8_lossy(&output.stdout);
                            debug!("  ✓ Found via 'where': {}", path.trim());
                        }
                        found
                    })
                    .unwrap_or(false);

                if where_result {
                    return true;
                }

                false
            }
            _ => false,
        };

        found
    }

    #[cfg(target_os = "macos")]
    {
        match browser {
            "firefox" => std::path::Path::new("/Applications/Firefox.app").exists(),
            "chrome" => std::path::Path::new("/Applications/Google Chrome.app").exists(),
            "edge" => std::path::Path::new("/Applications/Microsoft Edge.app").exists(),
            _ => false,
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // On Linux, check if the browser command is available
        Command::new("which")
            .arg(browser)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Map quality string to yt-dlp format selector
fn get_quality_format(quality: &str) -> String {
    match quality.to_lowercase().as_str() {
        "best" => {
            "bestvideo[ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string()
        }
        "1080p" | "1080" => {
            "bestvideo[height<=1080][ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]"
                .to_string()
        }
        "720p" | "720" => {
            "bestvideo[height<=720][ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]"
                .to_string()
        }
        "480p" | "480" => {
            "bestvideo[height<=480][ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]"
                .to_string()
        }
        "360p" | "360" => {
            "bestvideo[height<=360][ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]".to_string()
        }
        _ => {
            warn!("Unknown quality '{}', using 'best'", quality);
            "bestvideo[ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string()
        }
    }
}

/// Build arguments for yt-dlp based on download type
fn build_ytdlp_args(
    url: &str,
    output_path: &str,
    download_type: &DownloadType,
    browser_config: &BrowserConfig,
    binary_manager: &BinaryManager,
) -> Vec<String> {
    let mut args = vec![
        url.to_string(),
        "--no-playlist".to_string(),
        // Enable the Deno JS runtime for YouTube extraction (required since yt-dlp 2025+).
        // Deno is auto-downloaded by BinaryManager and its directory is injected into PATH
        // below, so it is available even when the end user has no system JS runtime.
        "--js-runtimes".to_string(),
        "deno".to_string(),
    ];

    // Add ffmpeg location - try bundled first, then system ffmpeg
    let ffmpeg_dir = match binary_manager.get_binary_path("ffmpeg") {
        Ok(ffmpeg_path) if ffmpeg_path.exists() => {
            ffmpeg_path.parent().map(|p| p.to_path_buf())
        }
        _ => None,
    };

    // If no bundled ffmpeg, check for system ffmpeg
    let ffmpeg_dir = ffmpeg_dir.or_else(|| {
        // Check common system locations
        let system_paths = [
            "/usr/bin",
            "/usr/local/bin",
            "/opt/homebrew/bin",  // macOS Homebrew
        ];

        for path in &system_paths {
            let ffmpeg_path = std::path::Path::new(path).join("ffmpeg");
            if ffmpeg_path.exists() {
                info!("✓ Found system ffmpeg at: {}", path);
                return Some(std::path::PathBuf::from(*path));
            }
        }

        // On Windows, check PATH
        #[cfg(target_os = "windows")]
        {
            if let Ok(path_var) = std::env::var("PATH") {
                for path in path_var.split(';') {
                    let ffmpeg_path = std::path::Path::new(path).join("ffmpeg.exe");
                    if ffmpeg_path.exists() {
                        info!("✓ Found system ffmpeg at: {}", path);
                        return Some(std::path::PathBuf::from(path));
                    }
                }
            }
        }

        None
    });

    if let Some(dir) = ffmpeg_dir {
        let ffmpeg_path_str = strip_extended_path_prefix(&dir);
        args.push("--ffmpeg-location".to_string());
        args.push(ffmpeg_path_str.clone());
        info!("✓ Using ffmpeg from: {}", ffmpeg_path_str);
    } else {
        warn!("⚠ ffmpeg not found - audio conversion may not work");
    }

    // Add format-specific arguments
    match download_type {
        DownloadType::Video { quality } => {
            args.push("-f".to_string());
            args.push(get_quality_format(quality));
            args.push("--merge-output-format".to_string());
            args.push("mp4".to_string());
        }
        DownloadType::Audio => {
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
            args.push("--audio-quality".to_string());
            args.push("0".to_string());
            args.push("--embed-thumbnail".to_string());
            args.push("--add-metadata".to_string());
        }
    }

    // Add browser cookie support if enabled
    if browser_config.use_cookies {
        if let Some(browser) = &browser_config.browser {
            args.push("--cookies-from-browser".to_string());
            args.push(browser.clone());
            info!("Using cookies from browser: {}", browser);
        } else {
            warn!("Browser cookies requested but no browser detected");
        }
    }

    // Add output path and progress options
    args.push("-o".to_string());
    args.push(output_path.to_string());
    args.push("--progress".to_string());
    args.push("--newline".to_string());

    args
}

/// Parse progress information from yt-dlp output
fn parse_progress(line: &str) -> Option<DownloadProgress> {
    if !line.contains("[download]") || !line.contains("%") {
        return None;
    }

    let percent_regex = Regex::new(r"(\d+(?:\.\d+)?)%").ok()?;
    let percent = percent_regex
        .captures(line)?
        .get(1)?
        .as_str()
        .parse::<f32>()
        .ok()?;

    let speed_regex = Regex::new(r"at\s+(\S+)").ok()?;
    let speed = speed_regex
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "---".to_string());

    let eta_regex = Regex::new(r"ETA\s+(\S+)").ok()?;
    let eta = eta_regex
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "--:--".to_string());

    Some(DownloadProgress {
        percent,
        speed,
        eta,
    })
}

/// Unified download function for both video and audio
pub async fn download_content(
    url: String,
    output_path: String,
    download_type: DownloadType,
    browser_config: BrowserConfig,
    window: tauri::WebviewWindow,
    app: AppHandle,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, DownloadHandle>>>,
    binary_manager: Arc<BinaryManager>,
) -> Result<String, DownloadError> {
    let download_id = Uuid::new_v4().to_string();

    info!(
        "Starting download: id={}, type={:?}, url={}, output={}",
        download_id, download_type, url, output_path
    );

    // Build arguments
    let args = build_ytdlp_args(&url, &output_path, &download_type, &browser_config, &binary_manager);
    debug!("yt-dlp args prepared (count: {})", args.len());

    // Get yt-dlp path from binary manager
    let ytdlp_path = binary_manager.get_binary_path("yt-dlp").ok();

    // Get the binaries directory (where the Deno JS runtime lives) to add to PATH.
    // yt-dlp discovers the runtime via PATH, and GUI-launched apps (especially on
    // macOS) do not inherit the user's shell PATH, so this must be set explicitly.
    let binaries_dir = binary_manager
        .get_binary_path("deno")
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Build modified PATH with our binaries directory prepended
    let modified_path = if let Some(ref bin_dir) = binaries_dir {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let bin_dir_str = bin_dir.to_string_lossy();
        #[cfg(target_os = "windows")]
        let new_path = format!("{};{}", bin_dir_str, current_path);
        #[cfg(not(target_os = "windows"))]
        let new_path = format!("{}:{}", bin_dir_str, current_path);
        info!("✓ Added bundled binaries to PATH: {}", bin_dir_str);
        new_path
    } else {
        std::env::var("PATH").unwrap_or_default()
    };

    // Spawn yt-dlp process (Deno JS runtime resolved via the modified PATH)
    let (mut rx, child) = if let Some(path) = ytdlp_path {
        if path.exists() {
            info!("Using downloaded yt-dlp from: {:?}", path);
            app.shell()
                .command(path)
                .args(&args)
                .env("PATH", &modified_path)
                .spawn()
                .map_err(|e| DownloadError::ProcessFailed(e.to_string()))?
        } else {
            info!("Using bundled yt-dlp sidecar (downloaded binary not found)");
            app.shell()
                .sidecar("yt-dlp")
                .map_err(|e| DownloadError::Sidecar(e.to_string()))?
                .args(&args)
                .env("PATH", &modified_path)
                .spawn()
                .map_err(|e| DownloadError::ProcessFailed(e.to_string()))?
        }
    } else {
        info!("Using bundled yt-dlp sidecar");
        app.shell()
            .sidecar("yt-dlp")
            .map_err(|e| DownloadError::Sidecar(e.to_string()))?
            .args(&args)
            .env("PATH", &modified_path)
            .spawn()
            .map_err(|e| DownloadError::ProcessFailed(e.to_string()))?
    };

    // Store download handle for potential cancellation
    {
        let mut downloads = active_downloads.lock().await;
        downloads.insert(
            download_id.clone(),
            DownloadHandle {
                id: download_id.clone(),
                child,
                url: url.clone(),
                output_path: output_path.clone(),
            },
        );
        info!("Stored download handle: {}", download_id);
    }

    // Emit download started event
    window
        .emit(
            "download-started",
            serde_json::json!({
                "id": download_id,
                "path": output_path
            }),
        )
        .ok();

    // Clone variables for async task
    let window_clone = window.clone();
    let window_clone2 = window.clone();
    let window_clone3 = window.clone();
    let output_path_clone = output_path.clone();
    let download_id_clone = download_id.clone();
    let active_downloads_clone = active_downloads.clone();

    // Spawn async task to handle command events
    tauri::async_runtime::spawn(async move {
        let mut stderr_buffer = String::new();

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_data) => {
                    let line = String::from_utf8_lossy(&line_data).to_string();
                    debug!("[stdout] {}", line);

                    // Detect processing phase (video merging or audio extraction)
                    if line.contains("[Merger]")
                        || line.contains("Merging formats")
                        || line.contains("[ffmpeg]")
                        || line.contains("[ExtractAudio]")
                        || line.contains("[FixupM4a]")
                        || line.contains("Post-process")
                    {
                        info!("Processing phase detected: {}", line.trim());
                        let message = if line.contains("[ExtractAudio]") || line.contains("audio") {
                            "Converting to MP3..."
                        } else {
                            "Processing video..."
                        };
                        window_clone
                            .emit(
                                "download-processing",
                                serde_json::json!({
                                    "message": message,
                                    "id": download_id_clone
                                }),
                            )
                            .ok();
                    }

                    // Parse and emit progress
                    if let Some(progress) = parse_progress(&line) {
                        window_clone.emit("download-progress", &progress).ok();
                    }
                }
                CommandEvent::Stderr(line_data) => {
                    let line = String::from_utf8_lossy(&line_data).to_string();
                    debug!("[stderr] {}", line);
                    stderr_buffer.push_str(&line);
                    stderr_buffer.push('\n');

                    // Emit status messages for important events
                    if line.contains("Sleeping") || line.contains("rate limit") {
                        window_clone2.emit("download-status", &line).ok();
                    }
                }
                CommandEvent::Terminated(payload) => {
                    // Remove from active downloads
                    {
                        let mut downloads = active_downloads_clone.lock().await;
                        downloads.remove(&download_id_clone);
                        info!("Removed download handle: {}", download_id_clone);
                    }

                    // Check if output file exists - yt-dlp may return non-zero even when file downloaded successfully
                    // This handles cases where post-processing has warnings but download succeeded
                    // Also find the actual file path (may have different extension than requested)
                    let output_path = std::path::Path::new(&output_path_clone);
                    let actual_file_path: Option<String> = if output_path.exists() {
                        Some(output_path_clone.clone())
                    } else {
                        // For audio downloads, yt-dlp might name the file differently
                        // Check for common variations: .mp3, .m4a, .webm, .opus, etc.
                        let base = output_path.with_extension("");
                        let base_str = base.to_string_lossy();
                        let extensions = ["mp3", "m4a", "webm", "opus", "mp4", "mkv", "aac", "ogg", "wav"];
                        extensions.iter().find_map(|ext| {
                            let variant = format!("{}.{}", base_str, ext);
                            if std::path::Path::new(&variant).exists() {
                                info!("Found actual file at: {}", variant);
                                Some(variant)
                            } else {
                                None
                            }
                        })
                    };

                    if let Some(code) = payload.code {
                        // SUCCESS: Either exit code 0 OR file exists (download succeeded despite warnings)
                        if code == 0 || actual_file_path.is_some() {
                            let final_path = actual_file_path.unwrap_or_else(|| output_path_clone.clone());
                            if code != 0 {
                                warn!("Download had non-zero exit code ({}) but file exists at {} - treating as success", code, final_path);
                            }
                            info!("Download completed successfully: {} -> {}", download_id_clone, final_path);
                            window_clone3
                                .emit(
                                    "download-complete",
                                    serde_json::json!({
                                        "success": true,
                                        "id": download_id_clone,
                                        "path": final_path
                                    }),
                                )
                                .ok();
                        } else {
                            // Log full stderr for debugging
                            error!(
                                "Download failed with exit code {}. Full stderr output:",
                                code
                            );
                            error!("{}", stderr_buffer);

                            // Analyze stderr to provide better error messages
                            let error_msg = if is_ffmpeg_error(&stderr_buffer) {
                                "Video processing failed. FFmpeg is required to merge video and audio streams. Please restart the application and try again.".to_string()
                            } else if is_dpapi_error(&stderr_buffer) {
                                "Cookie decryption failed. Chrome/Edge on Windows have encryption issues. Solutions: 1) Close your browser completely and try again, 2) Install Firefox (recommended), or 3) Disable browser cookies in settings.".to_string()
                            } else if is_auth_error(&stderr_buffer) {
                                "Authentication required. Try enabling browser cookies.".to_string()
                            } else if is_rate_limit_error(&stderr_buffer) {
                                "Rate limit exceeded. Please wait and try again.".to_string()
                            } else if is_network_error(&stderr_buffer) {
                                "Network error. Check your connection and try again.".to_string()
                            } else {
                                format!("Exit code: {}", code)
                            };

                            error!("Download failed: {} - {}", download_id_clone, error_msg);
                            window_clone3
                                .emit(
                                    "download-complete",
                                    serde_json::json!({
                                        "success": false,
                                        "id": download_id_clone,
                                        "error": error_msg
                                    }),
                                )
                                .ok();
                        }
                    } else {
                        error!(
                            "Download terminated without exit code: {}",
                            download_id_clone
                        );
                        window_clone3
                            .emit(
                                "download-complete",
                                serde_json::json!({
                                    "success": false,
                                    "id": download_id_clone,
                                    "error": "Process terminated without exit code"
                                }),
                            )
                            .ok();
                    }
                }
                _ => {}
            }
        }
    });

    Ok(download_id)
}

/// Smart download with automatic cookie retry
/// Attempts download without cookies first, then retries with cookies if authentication is needed
pub async fn download_content_with_smart_retry(
    url: String,
    output_path: String,
    download_type: DownloadType,
    window: tauri::WebviewWindow,
    app: AppHandle,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, DownloadHandle>>>,
    binary_manager: Arc<BinaryManager>,
) -> Result<String, DownloadError> {
    info!("🔄 Smart download initiated for: {}", url);

    // Attempt 1: Try WITHOUT cookies (works for 90% of videos)
    info!("📥 Attempt 1: Downloading without authentication...");
    let browser_config = BrowserConfig {
        use_cookies: false,
        browser: None,
    };

    match download_content(
        url.clone(),
        output_path.clone(),
        download_type.clone(),
        browser_config,
        window.clone(),
        app.clone(),
        active_downloads.clone(),
        binary_manager.clone(),
    )
    .await
    {
        Ok(download_id) => {
            info!("✅ Download succeeded without authentication!");
            return Ok(download_id);
        }
        Err(e) => {
            // Check if error is authentication-related
            let error_str = e.to_string();
            if error_str.contains("Authentication required")
                || error_str.contains("Sign in")
                || error_str.contains("Private video")
                || error_str.contains("login required")
                || error_str.contains("members-only")
            {
                warn!("🔐 Authentication required, retrying with browser cookies...");
            } else {
                // Not an auth error, fail immediately
                error!("❌ Download failed (not auth-related): {}", e);
                return Err(e);
            }
        }
    }

    // Attempt 2-4: Try with cookies from different browsers
    let browsers_to_try = vec!["firefox", "chrome", "edge"];

    for (index, browser_name) in browsers_to_try.iter().enumerate() {
        info!(
            "📥 Attempt {}: Trying with {} cookies...",
            index + 2,
            browser_name
        );

        // Check if browser is installed
        if !is_browser_installed(browser_name) {
            info!("⏭️  {} not installed, skipping...", browser_name);
            continue;
        }

        let browser_config = BrowserConfig {
            use_cookies: true,
            browser: Some(browser_name.to_string()),
        };

        match download_content(
            url.clone(),
            output_path.clone(),
            download_type.clone(),
            browser_config,
            window.clone(),
            app.clone(),
            active_downloads.clone(),
            binary_manager.clone(),
        )
        .await
        {
            Ok(download_id) => {
                info!("✅ Download succeeded with {} cookies!", browser_name);
                return Ok(download_id);
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("DPAPI") || error_str.contains("decrypt") {
                    warn!(
                        "⚠️  {} cookie decryption failed (DPAPI issue), trying next browser...",
                        browser_name
                    );
                    continue;
                } else {
                    // Different error, might be the actual problem
                    error!("❌ Download failed with {}: {}", browser_name, e);
                    // Try next browser anyway
                    continue;
                }
            }
        }
    }

    // All attempts failed
    error!("❌ All download attempts failed");
    Err(DownloadError::Authentication(
        "Unable to download this video. It may require login. Please verify the video is accessible in your browser, or install Firefox and log into the website there for automatic authentication.".to_string()
    ))
}

/// Cancel an active download
pub async fn cancel_download(
    download_id: String,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, DownloadHandle>>>,
    window: tauri::WebviewWindow,
) -> Result<(), DownloadError> {
    info!("Cancelling download: {}", download_id);

    let download_handle = {
        let mut downloads = active_downloads.lock().await;
        downloads.remove(&download_id)
    };

    if let Some(handle) = download_handle {
        // Kill the process
        handle
            .child
            .kill()
            .map_err(|e| DownloadError::ProcessFailed(format!("Failed to kill process: {}", e)))?;

        info!("Killed download process: {}", download_id);

        // Clean up temporary files (yt-dlp creates .part files)
        let part_file = format!("{}.part", handle.output_path);
        if std::path::Path::new(&part_file).exists() {
            std::fs::remove_file(&part_file).ok();
            info!("Cleaned up temp file: {}", part_file);
        }

        // Emit cancellation event
        window
            .emit(
                "download-cancelled",
                serde_json::json!({
                    "id": download_id,
                    "path": handle.output_path
                }),
            )
            .ok();

        Ok(())
    } else {
        warn!("Download not found: {}", download_id);
        Err(DownloadError::Unknown(format!(
            "Download not found: {}",
            download_id
        )))
    }
}
