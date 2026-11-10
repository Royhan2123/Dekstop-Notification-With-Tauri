//! Model domain. Field akan bertambah seiring fitur (review, kesehatan).
pub mod review;
pub mod settings;

pub use review::ReviewReport;

/// Satu sesi media dari OS. `session_index`/`session_count` diisi `MediaService`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaInfo {
    pub session_id: String,
    pub title: String,
    pub artist: String,
    pub playing: bool,
    /// Posisi saat sesi dibaca (sudah ditambah waktu berjalan).
    pub position_ms: u32,
    /// 0 bila tidak diketahui (mis. siaran langsung).
    pub duration_ms: u32,
    pub rate: f64,
    /// Data URL gambar sampul (<= 256 px).
    pub thumbnail: Option<String>,
    /// Warna dominan sampul, "#rrggbb".
    pub color: Option<String>,
    pub session_index: u32,
    pub session_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaCommand {
    Previous,
    TogglePlay,
    Next,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct HealthAlert {
    pub message: String,
}
