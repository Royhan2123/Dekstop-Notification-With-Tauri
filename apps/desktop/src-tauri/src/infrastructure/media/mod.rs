pub mod artwork;
#[cfg(target_os = "macos")]
pub mod macos_music;
#[cfg(windows)]
pub mod windows_smtc;

use std::sync::Arc;

use crate::domain::error::MediaError;
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::ports::MediaSource;

/// Adapter media untuk OS saat ini.
pub fn default_source() -> Arc<dyn MediaSource> {
    #[cfg(windows)]
    return Arc::new(windows_smtc::WindowsSmtc::default());
    // macOS: Music.app dan Spotify lewat AppleScript (MediaRemote dibatasi Apple sejak 15.4).
    #[cfg(target_os = "macos")]
    return Arc::new(macos_music::MacMedia);
    // shortcut: Linux (MPRIS) belum ada adapter, pulau tidak menampilkan media di sana.
    #[cfg(not(any(windows, target_os = "macos")))]
    return Arc::new(UnsupportedMediaSource);
}

#[cfg_attr(any(windows, target_os = "macos"), allow(dead_code))]
pub struct UnsupportedMediaSource;

impl MediaSource for UnsupportedMediaSource {
    fn start(&self, _: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
        tracing::warn!("sumber media belum didukung di sistem ini");
    }

    fn control(&self, _: &str, _: MediaCommand) -> Result<(), MediaError> {
        Err(MediaError::Unsupported)
    }
}

/// Posisi saat ini: posisi terakhir yang dilaporkan + waktu berjalan x kecepatan (bila diputar).
pub fn position_now(
    position_ms: i64,
    elapsed_ms: i64,
    rate: f64,
    playing: bool,
    duration_ms: i64,
) -> u32 {
    let mut pos = position_ms;
    if playing && elapsed_ms > 0 {
        pos += (elapsed_ms as f64 * rate) as i64;
    }
    if duration_ms > 0 {
        pos = pos.min(duration_ms);
    }
    pos.clamp(0, u32::MAX as i64) as u32
}

#[cfg(test)]
mod tests {
    use super::position_now;

    #[test]
    fn posisi_bertambah_saat_diputar_dan_dibatasi_durasi() {
        assert_eq!(position_now(1_000, 2_000, 1.0, true, 60_000), 3_000);
        assert_eq!(position_now(1_000, 2_000, 2.0, true, 60_000), 5_000);
        assert_eq!(position_now(1_000, 2_000, 1.0, false, 60_000), 1_000);
        assert_eq!(position_now(59_000, 5_000, 1.0, true, 60_000), 60_000);
        assert_eq!(position_now(1_000, -50, 1.0, true, 0), 1_000);
        assert_eq!(position_now(-5, 0, 1.0, true, 0), 0);
    }
}
