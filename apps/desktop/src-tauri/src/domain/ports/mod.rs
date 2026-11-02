//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use crate::domain::error::MediaError;
use crate::domain::island::state::IslandState;
use crate::domain::model::{MediaCommand, MediaInfo};

/// Penerima snapshot state pulau (mis. emit event ke frontend).
pub trait SnapshotSink: Send + Sync {
    fn publish(&self, state: &IslandState);
}

/// Sumber sesi media dari OS. Berbasis event, tanpa polling.
pub trait MediaSource: Send + Sync {
    /// Mulai mendengarkan; `on_change` dipanggil dengan daftar sesi terbaru setiap ada perubahan.
    fn start(&self, on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>);
    /// Blokir sampai perintah selesai; panggil dari thread non-UI.
    fn control(&self, session_id: &str, cmd: MediaCommand) -> Result<(), MediaError>;
}
