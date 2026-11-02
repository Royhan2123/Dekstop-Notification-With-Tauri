use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::domain::error::SettingsError;
use crate::domain::ports::Autostart;

pub struct TauriAutostart(pub AppHandle);

impl Autostart for TauriAutostart {
    fn set(&self, enabled: bool) -> Result<(), SettingsError> {
        // Build debug menunjuk ke exe di folder target; jangan daftarkan ke OS.
        if cfg!(debug_assertions) {
            tracing::info!("autostart={enabled} dilewati di build debug");
            return Ok(());
        }
        let manager = self.0.autolaunch();
        if manager.is_enabled().unwrap_or(false) == enabled {
            return Ok(());
        }
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|e| SettingsError::Storage(e.to_string()))
    }
}
