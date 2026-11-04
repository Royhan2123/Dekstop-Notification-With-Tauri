use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::domain::error::StartupError;
use crate::domain::ports::AppLauncher;

/// Membuka aplikasi/berkas lewat shell OS, sama seperti klik dua kali di Explorer.
pub struct ShellLauncher(pub AppHandle);

impl AppLauncher for ShellLauncher {
    fn launch(&self, path: &str) -> Result<(), StartupError> {
        self.0
            .opener()
            .open_path(path, None::<&str>)
            .map_err(|e| StartupError::Launch(e.to_string()))
    }
}
