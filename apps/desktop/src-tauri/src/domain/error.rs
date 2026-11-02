#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("tidak ada sesi media yang aktif")]
    NoSession,
    #[error("kontrol media belum didukung di sistem ini")]
    Unsupported,
    #[error("gagal mengontrol media: {0}")]
    Failed(String),
}
