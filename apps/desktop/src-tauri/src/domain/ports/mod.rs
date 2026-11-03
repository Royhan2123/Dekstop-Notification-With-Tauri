//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use std::path::Path;
use std::sync::Arc;

use crate::domain::bridge::BridgeCommand;
use crate::domain::error::{AiError, BridgeError, GitError, MediaError, SettingsError, UsageError};
use crate::domain::health::DayUsage;
use crate::domain::island::state::IslandState;
use crate::domain::model::review::{Diff, PushRef, SecretHit};
use crate::domain::model::settings::{AiProvider, Settings};
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

/// Operasi git untuk review. Semua metode memblokir; panggil dari thread non-UI.
pub trait GitRepo: Send + Sync {
    /// Diff dari semua commit yang akan di-push, tanpa lockfile dan pola `.islaignore`.
    fn diff_for_push(&self, repo: &Path, refs: &[PushRef]) -> Result<Diff, GitError>;
    /// Normalisasi dan validasi `path` sebagai repo git; mengembalikan akar repo.
    fn toplevel(&self, path: &Path) -> Result<String, GitError>;
    fn check_patch(&self, repo: &Path, patch: &str) -> Result<(), GitError>;
    /// Terapkan patch lalu commit; menolak bila ada perubahan staged lain.
    fn apply_and_commit(&self, repo: &Path, patch: &str, message: &str) -> Result<(), GitError>;
}

/// Pemasang git hook `pre-push` yang memanggil binary Isla.
pub trait HookInstaller: Send + Sync {
    fn install(&self, repo: &Path) -> Result<(), GitError>;
    fn uninstall(&self, repo: &Path) -> Result<(), GitError>;
}

/// Pemindai rahasia lokal (tanpa AI) di baris yang ditambahkan.
pub trait SecretScanner: Send + Sync {
    fn scan(&self, diff: &str) -> Vec<SecretHit>;
}

/// Otak AI: satu pertanyaan, satu jawaban teks. Prompt dan parsing ada di `application/review`.
pub trait AiClient: Send + Sync {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError>;
}

pub trait AiProviderFactory: Send + Sync {
    fn build(&self, provider: AiProvider, model: &str) -> Result<Arc<dyn AiClient>, AiError>;
}

/// Waktu sejak input terakhir (keyboard/mouse). `None` bila sistem tidak bisa mengetahuinya.
pub trait IdleDetector: Send + Sync {
    fn idle_seconds(&self) -> Option<u64>;
}

/// Kecerahan layar internal 0-100; `None` bila tidak tersedia (mis. monitor eksternal).
pub trait BrightnessReader: Send + Sync {
    fn read(&self) -> Option<u8>;
}

/// Volume utama dan status bisu; `None` bila tidak ada perangkat keluaran atau OS tidak mendukung.
pub trait VolumeReader: Send + Sync {
    fn read(&self) -> Option<crate::domain::island::state::VolumeInfo>;
}

/// Baca/tulis teks clipboard sistem.
pub trait ClipboardAccess: Send + Sync {
    fn read_text(&self) -> Option<String>;
    fn write_text(&self, text: &str) -> bool;
}

/// Keadaan daya laptop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerStatus {
    pub plugged: bool,
    pub percent: Option<u8>,
}

/// Status charger/baterai; `None` bila tidak ada baterai atau OS tidak mendukung.
pub trait PowerReader: Send + Sync {
    fn status(&self) -> Option<PowerStatus>;
}

/// Catatan pemakaian per menit.
pub trait UsageStore: Send + Sync {
    fn record(
        &self,
        date: &str,
        minute: u32,
        active: bool,
        brightness: Option<u8>,
    ) -> Result<(), UsageError>;
    /// Ringkasan harian untuk `dates` (urutan sama; hari kosong tetap ada dengan nol).
    fn days(&self, dates: &[String]) -> Result<Vec<DayUsage>, UsageError>;
}

/// Saluran ke ekstensi browser. `send` gagal bila belum ada ekstensi yang tersambung.
pub trait BrowserBridge: Send + Sync {
    fn send(&self, cmd: &BridgeCommand) -> Result<(), BridgeError>;
    /// Putuskan sambungan yang ada (mis. setelah pairing dicabut).
    fn disconnect(&self);
}

/// Keputusan aplikasi atas sambungan yang masuk dari ekstensi.
pub trait BridgeHandler: Send + Sync {
    /// Tukar kode pairing dengan token; `None` bila kode salah/kedaluwarsa.
    fn pair(&self, origin: &str, code: &str) -> Option<String>;
    fn authenticate(&self, origin: &str, token: &str) -> bool;
    fn connection_changed(&self, connected: bool);
}
