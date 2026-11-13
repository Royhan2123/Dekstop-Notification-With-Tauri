use crate::domain::model::{HealthAlert, MediaInfo, ReviewId, ReviewReport};

/// Isi yang bisa tampil di pulau. Data media disimpan di `IslandState::media`.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Media,
    ReviewRunning(ReviewId),
    ReviewResult(ReviewReport),
    HealthAlert(HealthAlert),
    Notice(String),
    /// Quick Ask: kotak pertanyaan ke AI.
    Ask(AskPhase),
    /// HUD volume: muncul sebentar saat volume atau bisu berubah.
    Volume(VolumeInfo),
    /// HUD kecerahan layar (0-100), seumur HUD volume.
    Brightness(u8),
    /// Rak file: daftar berkas di `IslandState::shelf`.
    Shelf,
    /// Jarvis sedang mendengarkan, berpikir, atau berbicara; tampil selama fase itu berlangsung.
    Voice(VoicePhase),
    /// Izin dari Jarvis (agen ingin menjalankan sesuatu): kartu dengan tombol Setuju/Tolak.
    Approval(Approval),
}

/// Permintaan izin dari Jarvis; `id` dipakai untuk menjawab lewat API lokal Jarvis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approval {
    pub id: String,
    pub text: String,
}

/// Fase asisten suara Jarvis (dikirim lewat IPC); "diam" berarti tidak ada fase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoicePhase {
    Listening,
    Thinking,
    Speaking,
}

impl VoicePhase {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "listening" => Some(Self::Listening),
            "thinking" => Some(Self::Thinking),
            "speaking" => Some(Self::Speaking),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listening => "listening",
            Self::Thinking => "thinking",
            Self::Speaking => "speaking",
        }
    }
}

/// Batas isi rak; yang paling lama dibuang saat penuh.
pub const SHELF_MAX: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumeInfo {
    /// 0-100.
    pub level: u8,
    pub muted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AskPhase {
    Input,
    Thinking,
    Done(String),
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum View {
    #[default]
    Idle,
    Compact(Content),
    Expanded(Content),
}

impl View {
    pub fn content(&self) -> Option<&Content> {
        match self {
            View::Idle => None,
            View::Compact(c) | View::Expanded(c) => Some(c),
        }
    }
}

/// Struct (bukan enum seperti di doc) karena antrian, media, dan flag ikut disimpan.
/// Tidak ada `Hidden` di `View`: pulau tersembunyi bila `fullscreen` true.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IslandState {
    pub view: View,
    pub queue: Vec<Content>,
    pub media: Option<MediaInfo>,
    pub hovered: bool,
    pub fullscreen: bool,
    pub dnd: bool,
    /// Ekstensi Isla Link sedang tersambung (tombol kecepatan/cari tampil).
    pub extension: bool,
    /// Path berkas di rak (terlama dulu); tetap ada walau pulau menampilkan hal lain.
    pub shelf: Vec<String>,
    /// Fase Jarvis sekarang; ditampilkan hanya bila pulau sedang kosong atau menampilkan media/HUD.
    pub voice: Option<VoicePhase>,
    pub token: u64,
}

/// Konfigurasi reducer.
pub struct Ctx {
    pub max_queue: usize,
    pub auto_dismiss_ms: u64,
    pub collapse_ms: u64,
    pub hud_ms: u64,
    /// Kartu izin tanpa jawaban ditutup setelah ini (Jarvis juga menolak sendiri).
    pub approval_ms: u64,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            max_queue: 5,
            auto_dismiss_ms: 8_000,
            collapse_ms: 5_000,
            hud_ms: 1_500,
            approval_ms: 30_000,
        }
    }
}
