//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use std::path::Path;
use std::sync::Arc;

use crate::domain::error::{AiError, GitError, MediaError, SettingsError};
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
