#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("tidak ada sesi media yang aktif")]
    NoSession,
    #[error("kontrol media belum didukung di sistem ini")]
    Unsupported,
    #[error("gagal mengontrol media: {0}")]
    Failed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("API Key tidak valid")]
    InvalidKey,
    #[error("repo tidak valid: {0}")]
    InvalidRepo(String),
    #[error("gagal menyimpan: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git gagal: {0}")]
    Command(String),
    #[error("ada perubahan staged lain; commit atau stash dulu")]
    StagedChanges,
    #[error("patch tidak bisa diterapkan: {0}")]
    BadPatch(String),
    #[error("sudah ada hook pre-push lain di repo ini; Isla tidak menimpanya")]
    HookExists,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("{0}")]
    NotConfigured(String),
    #[error("AI tidak tersedia: {0}")]
    Unavailable(String),
    #[error("jawaban AI tidak bisa dibaca")]
    BadResponse,
}

#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("gagal menyimpan pemakaian: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LyricsError {
    #[error("layanan lirik tidak tersedia: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("token GitHub belum diisi")]
    NoToken,
    #[error("{0}")]
    Invalid(String),
    #[error("GitHub tidak bisa dihubungi: {0}")]
    Unavailable(String),
    #[error(transparent)]
    Secrets(#[from] SettingsError),
}

#[derive(Debug, thiserror::Error)]
pub enum CalendarError {
    #[error("kalender tidak bisa diunduh: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("tema tidak bisa dipakai: {0}")]
    Invalid(String),
    #[error("tema tidak bisa diunduh: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("judul tugas kosong")]
    EmptyTitle,
    #[error("terlalu banyak tugas, hapus yang selesai dulu")]
    TooMany,
    #[error("gagal menyimpan tugas: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("ekstensi Isla Link belum tersambung")]
    NotConnected,
    #[error("perintah tidak valid")]
    Invalid,
    #[error("gagal mengirim ke ekstensi: {0}")]
    Failed(String),
}
