use super::state::{AskPhase, VolumeInfo};
use crate::domain::model::{HealthAlert, MediaInfo, ReviewId, ReviewReport};

pub enum IslandAction {
    MediaChanged(Option<MediaInfo>),
    ReviewStarted(ReviewId),
    ReviewFinished(ReviewId, ReviewReport),
    HealthAlert(HealthAlert),
    Notice(String),
    Hover(bool),
    Clicked,
    /// Tombol Musik di dok: tampilkan kartu musik lebar menggantikan kartu lain (bila ada lagu).
    MediaShow,
    Dismissed,
    /// Timer dengan token tertentu habis; diabaikan bila token sudah usang.
    Timeout(u64),
    HideForFullscreen(bool),
    DoNotDisturb(bool),
    ExtensionConnected(bool),
    /// Pengguna memanggil Quick Ask (pintasan keyboard).
    /// Volume atau bisu berubah (dari tombol volume, mixer, dsb).
    VolumeChanged(VolumeInfo),
    /// Kecerahan layar internal berubah (tombol Fn, Pengaturan Windows, dsb).
    BrightnessChanged(u8),
    AskOpen,
    /// Kemajuan pertanyaan; diabaikan bila kotak Quick Ask sudah ditutup.
    AskUpdate(AskPhase),
    /// Berkas dijatuhkan ke pulau: masuk rak dan rak ditampilkan.
    ShelfAdd(Vec<String>),
    /// Tampilkan rak apa adanya.
    ShelfShow,
    ShelfRemove(String),
    ShelfClear,
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    StartTimer { token: u64, ms: u64 },
}
