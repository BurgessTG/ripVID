use crate::platform::{executable_name, set_executable, unix_timestamp};
use hex;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};

/// Binaries the app downloads on first launch, in the order they are spawned
const REQUIRED_BINARIES: [&str; 3] = ["yt-dlp", "ffmpeg", "ffprobe"];

/// Marks a checksum mismatch so callers can distinguish it from a failed checksum fetch
const CHECKSUM_MISMATCH_PREFIX: &str = "Checksum mismatch!";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub last_check: u64,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub binary: String,
    pub progress: f64,
    pub status: String,
}

#[derive(Clone)]
pub struct BinaryManager {
    app_handle: AppHandle,
    data_dir: PathBuf,
}

impl BinaryManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("binaries");

        Self {
            app_handle,
            data_dir,
        }
    }

    /// Ensure all binaries are present and up-to-date
    /// This is called on app startup
    pub async fn ensure_all_binaries(&self) -> Result<(), String> {
        info!("Ensuring all required binaries are present...");

        // Create data directory
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create binaries directory: {}", e))?;

        // Check each binary (yt-dlp, ffmpeg, ffprobe - Bun used for JS runtime instead of Deno)
        let mut missing = Vec::new();

        for name in REQUIRED_BINARIES {
            if !self.is_binary_present(name)? {
                missing.push(name);
            }
        }

        // If any are missing, download them (first run)
        if !missing.is_empty() {
            info!("First run detected. Downloading: {:?}", missing);
            self.emit_progress("setup", 0.0, "Downloading required tools...")?;

            // Download in parallel for speed
            let manager1 = self.clone_for_background();
            let manager2 = self.clone_for_background();
            let manager3 = self.clone_for_background();

            let handles = vec![
                tokio::spawn(async move { manager1.download_ytdlp().await }),
                tokio::spawn(async move { manager2.download_ffmpeg().await }),
                tokio::spawn(async move { manager3.download_ffprobe().await }),
            ];

            let mut errors = Vec::new();
            for (i, handle) in handles.into_iter().enumerate() {
                let binary_name = REQUIRED_BINARIES[i];
                match handle.await {
                    Ok(Ok(())) => {
                        info!("{} downloaded successfully", binary_name);
                    }
                    Ok(Err(e)) => {
                        error!("{} download failed: {}", binary_name, e);
                        errors.push(format!("{}: {}", binary_name, e));
                    }
                    Err(e) => {
                        error!("{} task panicked: {}", binary_name, e);
                        errors.push(format!("{}: task failed", binary_name));
                    }
                }
            }

            if !errors.is_empty() {
                return Err(format!("Failed to download: {}", errors.join(", ")));
            }

            self.emit_progress("setup", 100.0, "All tools ready!")?;
        }

        // Check for updates in background (non-blocking)
        let manager = self.clone_for_background();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = manager.check_updates_background().await {
                warn!("Background update check failed: {}", e);
            }
        });

        Ok(())
    }

    /// Check for updates in the background (once per day)
    async fn check_updates_background(&self) -> Result<(), String> {
        if !self.should_check_updates()? {
            info!("Skipping update check - checked recently");
            return Ok(());
        }

        info!("Checking for binary updates...");

        // Update each binary if needed
        log_update_result("yt-dlp", self.update_ytdlp_if_needed().await);
        log_update_result("ffmpeg", self.update_ffmpeg_if_needed().await);

        // Save last check time
        self.save_last_check()?;

        Ok(())
    }

    fn should_check_updates(&self) -> Result<bool, String> {
        let version_file = self.data_dir.join("last-check.json");

        if !version_file.exists() {
            return Ok(true);
        }

        let content = fs::read_to_string(&version_file).ok();
        if let Some(content) = content {
            if let Ok(last_check) = content.parse::<u64>() {
                // Check once per day (86400 seconds)
                return Ok(unix_timestamp() - last_check > 86400);
            }
        }

        Ok(true)
    }

    fn save_last_check(&self) -> Result<(), String> {
        let now = unix_timestamp();

        let version_file = self.data_dir.join("last-check.json");
        fs::write(version_file, now.to_string()).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Check if a binary is present
    fn is_binary_present(&self, name: &str) -> Result<bool, String> {
        let path = self.get_binary_path(name)?;
        Ok(path.exists())
    }

    /// Get the path for a binary (platform-aware)
    pub fn get_binary_path(&self, name: &str) -> Result<PathBuf, String> {
        Ok(self.data_dir.join(executable_name(name)))
    }

    /// Get the current version of a binary from saved info
    fn get_current_version(&self, name: &str) -> Option<String> {
        let info_file = self.data_dir.join(format!("{}-info.json", name));
        if !info_file.exists() {
            return None;
        }

        let content = fs::read_to_string(&info_file).ok()?;
        let info: BinaryInfo = serde_json::from_str(&content).ok()?;
        Some(info.version)
    }

    /// Download yt-dlp
    async fn download_ytdlp(&self) -> Result<(), String> {
        self.emit_progress("yt-dlp", 0.0, "Downloading yt-dlp...")?;

        let client = reqwest::Client::new();
        let release = self.fetch_latest_ytdlp_release(&client).await?;

        self.emit_progress("yt-dlp", 25.0, "Downloading binary...")?;
        let bytes = self.download_ytdlp_asset(&client, &release).await?;

        self.emit_progress("yt-dlp", 75.0, "Verifying checksum...")?;
        self.verify_ytdlp_checksum(&client, &release, &bytes).await?;

        let path = self.install_binary("yt-dlp", &bytes, &release.tag_name)?;

        self.emit_progress("yt-dlp", 100.0, "Ready!")?;

        info!(
            "yt-dlp {} installed successfully at {:?}",
            release.tag_name, path
        );

        Ok(())
    }

    /// Download ffmpeg with fallback sources
    async fn download_ffmpeg(&self) -> Result<(), String> {
        self.download_with_fallback_sources("ffmpeg", self.get_ffmpeg_sources())
            .await
    }

    /// Download ffprobe with fallback sources
    async fn download_ffprobe(&self) -> Result<(), String> {
        self.download_with_fallback_sources("ffprobe", self.get_ffprobe_sources())
            .await
    }

    /// Try each source in order until one yields a usable binary
    async fn download_with_fallback_sources(
        &self,
        binary_name: &str,
        sources: Vec<DownloadSource>,
    ) -> Result<(), String> {
        self.emit_progress(
            binary_name,
            0.0,
            &format!("Downloading {}...", binary_name),
        )?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 min timeout for large files
            .build()
            .map_err(|e| e.to_string())?;

        for (i, source) in sources.iter().enumerate() {
            info!(
                "Trying {} source {}/{}: {}",
                binary_name,
                i + 1,
                sources.len(),
                source.name
            );

            match self.download_from_source(&client, binary_name, source).await {
                Ok(()) => {
                    self.emit_progress(binary_name, 100.0, "Ready!")?;
                    info!(
                        "{} downloaded successfully from {}",
                        binary_name, source.name
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to download from {}: {}", source.name, e);
                    if i < sources.len() - 1 {
                        info!("Trying next source...");
                    }
                }
            }
        }

        Err(format!("All {} sources failed", binary_name))
    }

    async fn download_from_source(
        &self,
        client: &reqwest::Client,
        binary_name: &str,
        source: &DownloadSource,
    ) -> Result<(), String> {
        self.emit_progress(binary_name, 25.0, &format!("Downloading from {}...", source.name))?;

        let response = client
            .get(&source.url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        self.emit_progress(binary_name, 75.0, "Extracting binary...")?;

        // Handle archive extraction based on type
        let final_bytes = match source.archive_type {
            ArchiveType::None => bytes.to_vec(),
            ArchiveType::Zip => self.extract_from_zip(&bytes, binary_name)?,
            ArchiveType::TarXz => {
                let decoder = xz2::read::XzDecoder::new(std::io::Cursor::new(&bytes));
                self.extract_from_tar(decoder, binary_name, "tar.xz")?
            }
            ArchiveType::TarGz => {
                let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes));
                self.extract_from_tar(decoder, binary_name, "tar.gz")?
            }
        };

        self.install_binary(binary_name, &final_bytes, &source.version)?;

        Ok(())
    }

    /// Write a binary to its final location, make it executable and record its version
    fn install_binary(&self, name: &str, bytes: &[u8], version: &str) -> Result<PathBuf, String> {
        let path = self.get_binary_path(name)?;
        fs::write(&path, bytes).map_err(|e| format!("Failed to save: {}", e))?;
        set_executable(&path)?;
        self.save_binary_info(name, version, &path)?;

        Ok(path)
    }

    /// Extract a binary from a ZIP archive
    fn extract_from_zip(&self, bytes: &[u8], binary_name: &str) -> Result<Vec<u8>, String> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Invalid zip: {}", e))?;

        let target_name = executable_name(binary_name);

        // Look for the binary in the zip (may be in subdirectory)
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let file_name = file.name().to_string();

            // Check if this file matches our target (handle nested paths)
            let is_match = file_name.ends_with(&target_name)
                || file_name.ends_with(&format!("/{}", target_name))
                || file_name.ends_with(&format!("\\{}", target_name));

            if is_match && !file.is_dir() {
                let mut buffer = Vec::new();
                io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                info!("Extracted {} from zip ({})", binary_name, file_name);
                return Ok(buffer);
            }
        }

        Err(format!("{} not found in zip archive", target_name))
    }

    /// Extract a binary from a decompressed tar stream
    /// `label` only identifies the compression format in log messages.
    fn extract_from_tar<R: Read>(
        &self,
        reader: R,
        binary_name: &str,
        label: &str,
    ) -> Result<Vec<u8>, String> {
        info!("Extracting {} from {} archive...", binary_name, label);

        let mut archive = tar::Archive::new(reader);
        let entries = archive
            .entries()
            .map_err(|e| format!("Failed to read tar entries: {}", e))?;

        for entry_result in entries {
            let mut entry = entry_result.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path_str = {
                let path = entry.path().map_err(|e| format!("Failed to get path: {}", e))?;
                path.to_string_lossy().to_string()
            };

            // Archives are usually nested (e.g. ffmpeg-6.0-amd64-static/ffmpeg), and an exact
            // filename match avoids picking ffprobe when looking for ffmpeg
            let filename = std::path::Path::new(&path_str)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            if filename == binary_name && entry.header().entry_type().is_file() {
                let mut buffer = Vec::new();
                entry
                    .read_to_end(&mut buffer)
                    .map_err(|e| format!("Failed to read entry: {}", e))?;
                info!("Extracted {} from {} ({})", binary_name, label, path_str);
                return Ok(buffer);
            }
        }

        Err(format!("{} not found in {} archive", binary_name, label))
    }

    fn get_ytdlp_asset_name(&self) -> &str {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return "yt-dlp.exe";

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return "yt-dlp_macos";

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return "yt-dlp_macos";

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return "yt-dlp_linux";

        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "x86_64")
        )))]
        return "yt-dlp";
    }

    fn get_ffmpeg_sources(&self) -> Vec<DownloadSource> {
        #[cfg(target_os = "windows")]
        return vec![
            DownloadSource {
                name: "gyan.dev",
                url: "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::Zip,
            },
            DownloadSource {
                name: "BtbN/FFmpeg-Builds",
                url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::Zip,
            },
        ];

        #[cfg(target_os = "macos")]
        return vec![
            DownloadSource {
                name: "evermeet.cx",
                url: "https://evermeet.cx/ffmpeg/getrelease/zip".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::Zip,
            },
        ];

        #[cfg(target_os = "linux")]
        return vec![
            DownloadSource {
                name: "johnvansickle.com",
                url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::TarXz,
            },
            DownloadSource {
                name: "BtbN/FFmpeg-Builds",
                url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::TarXz,
            },
        ];
    }

    fn get_ffprobe_sources(&self) -> Vec<DownloadSource> {
        #[cfg(target_os = "windows")]
        return vec![
            DownloadSource {
                name: "gyan.dev",
                url: "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::Zip,
            },
        ];

        #[cfg(target_os = "macos")]
        return vec![
            DownloadSource {
                name: "evermeet.cx",
                url: "https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::Zip,
            },
        ];

        #[cfg(target_os = "linux")]
        return vec![
            // ffprobe is included in the ffmpeg static build
            DownloadSource {
                name: "johnvansickle.com",
                url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz".to_string(),
                version: "latest".to_string(),
                archive_type: ArchiveType::TarXz,
            },
        ];
    }

    /// Check and update yt-dlp if a newer version is available
    async fn update_ytdlp_if_needed(&self) -> Result<bool, String> {
        info!("Checking for yt-dlp updates...");

        let client = reqwest::Client::new();
        let release = self.fetch_latest_ytdlp_release(&client).await?;

        if let Some(current) = self.get_current_version("yt-dlp") {
            if current == release.tag_name {
                info!("yt-dlp is up to date ({})", current);
                return Ok(false);
            }
            info!(
                "yt-dlp update available: {} -> {}",
                current, release.tag_name
            );
        } else {
            info!("No yt-dlp version info found, will download latest");
        }

        info!("Downloading yt-dlp {}...", release.tag_name);
        let bytes = self.download_ytdlp_asset(&client, &release).await?;

        // A failed checksum fetch is tolerated when updating an already working install
        match self.verify_ytdlp_checksum(&client, &release, &bytes).await {
            Ok(()) => info!("Checksum verified for yt-dlp update"),
            Err(e) if e.starts_with(CHECKSUM_MISMATCH_PREFIX) => return Err(e),
            Err(e) => warn!("Could not verify checksum ({}), proceeding anyway", e),
        }

        // Back up the existing binary so a failed write can be rolled back
        let path = self.get_binary_path("yt-dlp")?;
        let backup_path = self
            .data_dir
            .join(format!("{}.backup", executable_name("yt-dlp")));

        if path.exists() {
            fs::copy(&path, &backup_path).ok();
        }

        if let Err(e) = self.install_binary("yt-dlp", &bytes, &release.tag_name) {
            if backup_path.exists() {
                fs::copy(&backup_path, &path).ok();
            }
            return Err(e);
        }

        if backup_path.exists() {
            fs::remove_file(&backup_path).ok();
        }

        info!("Successfully updated yt-dlp to {}", release.tag_name);
        Ok(true)
    }

    /// Fetch metadata for the latest yt-dlp GitHub release
    async fn fetch_latest_ytdlp_release(
        &self,
        client: &reqwest::Client,
    ) -> Result<GitHubRelease, String> {
        let response = client
            .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
            .header("User-Agent", "ripVID")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch yt-dlp release: {}", e))?;

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse release: {}", e))
    }

    /// Download the yt-dlp asset matching the current platform
    async fn download_ytdlp_asset(
        &self,
        client: &reqwest::Client,
        release: &GitHubRelease,
    ) -> Result<Vec<u8>, String> {
        let asset_name = self.get_ytdlp_asset_name();
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No asset found for {}", asset_name))?;

        let response = client
            .get(&asset.browser_download_url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| format!("Failed to read bytes: {}", e))
    }

    /// Compare downloaded bytes against the checksum published with the release
    async fn verify_ytdlp_checksum(
        &self,
        client: &reqwest::Client,
        release: &GitHubRelease,
        bytes: &[u8],
    ) -> Result<(), String> {
        let checksums_url = format!(
            "https://github.com/yt-dlp/yt-dlp/releases/download/{}/SHA2-256SUMS",
            release.tag_name
        );

        let expected_checksum = self
            .fetch_and_parse_checksum(client, &checksums_url, self.get_ytdlp_asset_name())
            .await?;

        let actual_checksum = self.calculate_sha256(bytes);

        if actual_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            return Err(format!(
                "{} Expected: {}, Got: {}",
                CHECKSUM_MISMATCH_PREFIX, expected_checksum, actual_checksum
            ));
        }

        Ok(())
    }

    /// Check and update ffmpeg if a newer version is available
    async fn update_ffmpeg_if_needed(&self) -> Result<bool, String> {
        info!("Checking for ffmpeg updates...");

        // ffmpeg doesn't have frequent breaking updates like yt-dlp
        // We check if the binary exists and is reasonably recent
        let ffmpeg_path = self.get_binary_path("ffmpeg")?;

        if !ffmpeg_path.exists() {
            info!("ffmpeg not found, downloading...");
            self.download_ffmpeg().await?;
            return Ok(true);
        }

        // Check file age - update if older than 30 days
        if let Ok(metadata) = fs::metadata(&ffmpeg_path) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();

                // 30 days in seconds
                if age.as_secs() > 30 * 24 * 60 * 60 {
                    info!("ffmpeg is older than 30 days, updating...");
                    self.download_ffmpeg().await?;
                    return Ok(true);
                }
            }
        }

        info!("ffmpeg is up to date");
        Ok(false)
    }

    fn save_binary_info(&self, name: &str, version: &str, path: &PathBuf) -> Result<(), String> {
        let info = BinaryInfo {
            name: name.to_string(),
            version: version.to_string(),
            last_check: unix_timestamp(),
            path: path.to_string_lossy().to_string(),
        };

        let info_file = self.data_dir.join(format!("{}-info.json", name));
        let json = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;

        fs::write(info_file, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    async fn fetch_and_parse_checksum(
        &self,
        client: &reqwest::Client,
        checksums_url: &str,
        asset_name: &str,
    ) -> Result<String, String> {
        let response = client
            .get(checksums_url)
            .header("User-Agent", "ripVID")
            .send()
            .await
            .map_err(|e| format!("Failed to download checksum file: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download checksum file: HTTP {}",
                response.status()
            ));
        }

        let checksums_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read checksum file: {}", e))?;

        for line in checksums_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0];
                let filename = parts[1];

                if filename == asset_name {
                    return Ok(hash.to_string());
                }
            }
        }

        Err(format!("Checksum not found for {}", asset_name))
    }

    fn emit_progress(&self, binary: &str, progress: f64, status: &str) -> Result<(), String> {
        let event = DownloadProgress {
            binary: binary.to_string(),
            progress,
            status: status.to_string(),
        };

        self.app_handle
            .emit("binary-download-progress", event)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn clone_for_background(&self) -> Self {
        self.clone()
    }
}

/// Log the outcome of a background update attempt
fn log_update_result(binary_name: &str, result: Result<bool, String>) {
    match result {
        Ok(true) => info!("{} was updated successfully", binary_name),
        Ok(false) => {}
        Err(e) => warn!("Failed to update {}: {}", binary_name, e),
    }
}

/// Type of archive for extraction
#[derive(Debug, Clone)]
enum ArchiveType {
    None,
    Zip,
    TarXz,
    TarGz,
}

/// Download source configuration
struct DownloadSource {
    name: &'static str,
    url: String,
    version: String,
    archive_type: ArchiveType,
}
