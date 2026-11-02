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
