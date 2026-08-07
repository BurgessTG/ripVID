// Shared platform helpers used by the binary manager, download pipeline and commands.

use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

/// Seconds elapsed since the Unix epoch
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Executable file name for a binary on the current platform
pub fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// Mark a file as executable. No-op on platforms without Unix permissions.
pub fn set_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

/// Open a file or folder with the operating system's default handler
pub fn open_with_os(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", &path.replace('/', "\\")]);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    match command.spawn() {
        Ok(_) => {
            info!("Opened path with system handler: {}", path);
            Ok(())
        }
        Err(e) => {
            error!("Failed to open path {}: {}", path, e);
            Err(format!("Failed to open path: {}", e))
        }
    }
}
