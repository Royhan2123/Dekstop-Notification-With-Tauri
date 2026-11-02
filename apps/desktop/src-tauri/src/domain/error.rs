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
    #[error("gagal menyimpan: {0}")]
    Storage(String),
}
