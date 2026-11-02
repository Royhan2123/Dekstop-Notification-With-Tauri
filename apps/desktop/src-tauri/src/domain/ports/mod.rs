//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use crate::domain::error::{MediaError, SettingsError};
use crate::domain::island::state::IslandState;
use crate::domain::model::settings::Settings;
use crate::domain::model::{MediaCommand, MediaInfo};

/// Penerima snapshot state pulau (mis. emit event ke frontend).
pub trait SnapshotSink: Send + Sync {
    fn publish(&self, state: &IslandState);
}

/// Penerima perubahan setelan (mis. emit event ke semua jendela).
pub trait SettingsSink: Send + Sync {
    fn publish(&self, settings: &Settings);
}

/// Sumber sesi media dari OS. Berbasis event, tanpa polling.
pub trait MediaSource: Send + Sync {
    /// Mulai mendengarkan; `on_change` dipanggil dengan daftar sesi terbaru setiap ada perubahan.
    fn start(&self, on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>);
    /// Blokir sampai perintah selesai; panggil dari thread non-UI.
    fn control(&self, session_id: &str, cmd: MediaCommand) -> Result<(), MediaError>;
}

pub trait SettingsRepository: Send + Sync {
    /// Setelan tersimpan, atau default bila belum ada / rusak.
    fn load(&self) -> Settings;
    fn save(&self, settings: &Settings) -> Result<(), SettingsError>;
}

/// Brankas rahasia OS (Keychain / Credential Manager / Secret Service). Nilai tidak pernah dilog.
pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError>;
    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError>;
    fn delete(&self, key: &str) -> Result<(), SettingsError>;
}

/// Mulai otomatis saat login.
pub trait Autostart: Send + Sync {
    fn set(&self, enabled: bool) -> Result<(), SettingsError>;
}
