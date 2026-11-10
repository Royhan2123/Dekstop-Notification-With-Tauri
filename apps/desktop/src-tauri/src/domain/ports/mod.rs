//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};

use crate::domain::bridge::{BridgeCommand, BrowserMedia};
use crate::domain::chat::ChatTurn;
use crate::domain::disk::DiskReport;
use crate::domain::error::{
    AiError, BridgeError, CalendarError, ChatError, DiskError, GitError, GithubError, LyricsError,
    MediaError, PowerError, SettingsError, StartupError, TaskError, ThemeError, UsageError,
};
use crate::domain::health::DayUsage;
use crate::domain::island::state::IslandState;
use crate::domain::lyrics::Candidate;
use crate::domain::model::review::{Diff, PushRef, SecretHit};
use crate::domain::model::settings::{AiProvider, Settings};
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::power::PowerAction;
use crate::domain::startup::{InstalledApp, StartupApp};
use crate::domain::task::Task;

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
    /// Diff perubahan yang sudah di-`git add`, dengan penyaringan yang sama seperti `diff_for_push`.
    fn staged_diff(&self, repo: &Path) -> Result<Diff, GitError>;
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

    /// Nama model yang bisa dipilih pengguna; kosong bila penyedia tidak punya daftar.
    fn models(&self) -> Result<Vec<String>, AiError> {
        Ok(Vec::new())
    }
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

/// Mengubah volume utama dan bisu. Dipakai saat Isla mengambil alih tombol volume.
pub trait VolumeControl: Send + Sync {
    /// Atur volume 0-100; `false` bila perangkat tidak mendukung.
    fn set_level(&self, level: u8) -> bool;
    fn set_muted(&self, muted: bool) -> bool;
}

/// Baca/tulis teks clipboard sistem.
pub trait ClipboardAccess: Send + Sync {
    fn read_text(&self) -> Option<String>;
    fn write_text(&self, text: &str) -> bool;
    /// Taruh daftar berkas di clipboard (bisa ditempel di Explorer atau aplikasi lain).
    fn write_files(&self, paths: &[String]) -> bool;
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

/// Perintah daya OS (matikan, mulai ulang, tidur, kunci).
pub trait PowerControl: Send + Sync {
    fn perform(&self, action: PowerAction) -> Result<(), PowerError>;
}

/// Sumber lirik berwaktu (pencarian teks bebas).
pub trait LyricsSource: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<Candidate>, LyricsError>;
}

/// Mengunduh isi berkas tema dari URL yang sudah lolos pemeriksaan.
pub trait ThemeFetcher: Send + Sync {
    fn fetch(&self, url: &str) -> Result<String, ThemeError>;
}

/// Mengunduh isi kalender iCal dari URL yang sudah lolos pemeriksaan.
pub trait CalendarSource: Send + Sync {
    fn fetch(&self, url: &str) -> Result<String, CalendarError>;
}

/// GitHub: PR yang menunggu review kita dan status CI PR kita, dengan token pribadi.
pub trait GithubApi: Send + Sync {
    fn radar(&self, token: &str) -> Result<crate::domain::github::Radar, GithubError>;
}

/// Wallpaper desktop saat ini sebagai data URL gambar kecil.
pub trait WallpaperSource: Send + Sync {
    fn current(&self) -> Option<String>;
}

/// Memutar bunyi notifikasi (tidak menunggu bunyinya selesai).
pub trait SoundPlayer: Send + Sync {
    fn chime(&self);
}

/// Apakah aplikasi layar penuh (game, video, presentasi) sedang aktif.
pub trait FullscreenDetector: Send + Sync {
    fn is_fullscreen(&self) -> bool;
}

/// Total byte jaringan fisik sejak boot (unduh, unggah); `None` bila OS tidak mendukung.
pub trait NetCounters: Send + Sync {
    fn totals(&self) -> Option<(u64, u64)>;
}

/// Penyimpanan riwayat Tanya AI.
pub trait ChatStore: Send + Sync {
    /// Dari yang terlama ke terbaru.
    fn list(&self) -> Result<Vec<ChatTurn>, ChatError>;
    /// Menambah satu giliran; yang melewati `MAX_CHAT_TURNS` terlama dibuang.
    fn add(&self, turn: &ChatTurn) -> Result<(), ChatError>;
    fn clear(&self) -> Result<(), ChatError>;
}

/// Penyimpanan daftar tugas.
pub trait TaskStore: Send + Sync {
    fn list(&self) -> Result<Vec<Task>, TaskError>;
    fn add(&self, title: &str, note: bool) -> Result<(), TaskError>;
    /// Ubah tugas menjadi catatan atau sebaliknya; judul dan isi tetap.
    fn set_note(&self, id: i32, note: bool) -> Result<(), TaskError>;
    fn update_note(&self, id: i32, title: &str, body: &str) -> Result<(), TaskError>;
    fn set_done(&self, id: i32, done: bool) -> Result<(), TaskError>;
    fn remove(&self, id: i32) -> Result<(), TaskError>;
    fn clear_done(&self) -> Result<(), TaskError>;
}

/// Penyimpanan daftar aplikasi startup (urut menurut posisi).
pub trait StartupStore: Send + Sync {
    fn list(&self) -> Result<Vec<StartupApp>, StartupError>;
    fn add(&self, name: &str, path: &str, delay_secs: u32) -> Result<(), StartupError>;
    fn set_delay(&self, id: i32, delay_secs: u32) -> Result<(), StartupError>;
    /// Posisi baru = urutan id pada `ids`.
    fn reorder(&self, ids: &[i32]) -> Result<(), StartupError>;
    fn remove(&self, id: i32) -> Result<(), StartupError>;
}

/// Daftar aplikasi terpasang (seperti Program dan Fitur di Control Panel), tiap entri punya .exe.
pub trait InstalledApps: Send + Sync {
    fn list(&self) -> Vec<InstalledApp>;
}

/// Membuka satu aplikasi/berkas seperti klik dua kali di Explorer.
pub trait AppLauncher: Send + Sync {
    fn launch(&self, path: &str) -> Result<(), StartupError>;
}

/// Menghitung ukuran isi folder. Hanya membaca; tidak boleh mengubah atau menghapus apa pun.
pub trait DiskScanner: Send + Sync {
    /// Akar yang bisa dipindai (drive di Windows).
    fn drives(&self) -> Vec<String>;
    /// Ukuran tiap isi langsung `root` (folder = total turunannya). Memblokir; berhenti dini bila `cancel` menyala.
    /// `progress` bertambah per berkas yang dihitung.
    fn scan(
        &self,
        root: &str,
        cancel: &AtomicBool,
        progress: &AtomicU64,
    ) -> Result<DiskReport, DiskError>;
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
    /// Daftar tab browser yang sedang memutar media (kosong = tidak ada / sambungan putus).
    fn media(&self, _sessions: Vec<BrowserMedia>) {}
}
