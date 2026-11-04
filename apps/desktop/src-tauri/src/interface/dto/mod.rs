//! DTO yang dikirim ke frontend. Dihasilkan dari state domain; tipe TS dibuat otomatis.
use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::application::disk_service::DiskStatus;
use crate::application::health_service::HealthSummary;
use crate::domain::health::DayUsage;
use crate::domain::island::state::{AskPhase, Content, IslandState, View};
use crate::domain::mascot::{Mood, mood_for};
use crate::domain::model::review::Severity;
use crate::domain::model::settings::{AiProvider, Settings};
use crate::domain::model::{MediaCommand, ReviewReport};
use crate::domain::power::PowerAction;
use crate::domain::startup::{InstalledApp, StartupApp};
use crate::domain::task::Task;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum CiDto {
    Passing,
    Failing,
    Pending,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct PrDto {
    pub repo: String,
    pub title: String,
    pub url: String,
    pub ci: CiDto,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct RadarDto {
    pub review_requests: Vec<PrDto>,
    pub mine: Vec<PrDto>,
}

impl From<&crate::domain::github::Radar> for RadarDto {
    fn from(r: &crate::domain::github::Radar) -> Self {
        use crate::domain::github::Ci;
        let pr = |p: &crate::domain::github::Pr| PrDto {
            repo: p.repo.clone(),
            title: p.title.clone(),
            url: p.url.clone(),
            ci: match p.ci {
                Ci::Passing => CiDto::Passing,
                Ci::Failing => CiDto::Failing,
                Ci::Pending => CiDto::Pending,
                Ci::Unknown => CiDto::Unknown,
            },
        };
        Self {
            review_requests: r.review_requests.iter().map(pr).collect(),
            mine: r.mine.iter().map(pr).collect(),
        }
    }
}

/// Rapat berikutnya untuk Setelan: judul, jam (HH:MM), dan tanggal (YYYY-MM-DD) menurut jam komputer.
/// Satu pertanyaan dan jawabannya di riwayat Tanya AI; `by` adalah AI yang menjawab.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct ChatTurnDto {
    pub question: String,
    pub answer: String,
    pub by: String,
}

/// Penyedia dan model yang menjawab Tanya AI; `model` kosong bila penyedia menentukannya sendiri.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct AiActiveDto {
    pub provider: AiProvider,
    pub model: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct MeetingDto {
    pub title: String,
    pub time: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum IslandMode {
    Hidden,
    Idle,
    Compact,
    Expanded,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Media,
    ReviewRunning,
    ReviewResult,
    HealthAlert,
    Notice,
    Ask,
    Volume,
    Brightness,
    Shelf,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct IslandContent {
    pub kind: ContentKind,
    pub text: String,
    pub detail: Option<String>,
}

/// Data tambahan saat pulau menampilkan media.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct MediaDto {
    pub playing: bool,
    pub position_ms: u32,
    pub duration_ms: u32,
    pub rate: f64,
    pub thumbnail: Option<String>,
    pub color: Option<String>,
    pub session_index: u32,
    pub session_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum MediaCommandDto {
    Previous,
    TogglePlay,
    Next,
}

impl From<MediaCommandDto> for MediaCommand {
    fn from(c: MediaCommandDto) -> Self {
        match c {
            MediaCommandDto::Previous => MediaCommand::Previous,
            MediaCommandDto::TogglePlay => MediaCommand::TogglePlay,
            MediaCommandDto::Next => MediaCommand::Next,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct IslandSnapshot {
    pub mode: IslandMode,
    pub content: Option<IslandContent>,
    pub media: Option<MediaDto>,
    /// Ada lagu yang bisa ditampilkan, walau pulau sedang menampilkan kartu lain (tombol Musik di dok).
    pub has_media: bool,
    /// Laporan lengkap bila pulau menampilkan hasil review.
    pub review: Option<ReviewDto>,
    /// Ekspresi Lumi untuk keadaan pulau saat ini.
    pub mood: Mood,
    pub extension_connected: bool,
    pub hovered: bool,
    /// Isi rak; hanya terisi saat pulau menampilkan rak.
    pub shelf: Vec<ShelfItemDto>,
    /// Jangan Ganggu aktif (menu klik kanan menampilkan centangnya).
    pub dnd: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct ShelfItemDto {
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct FindingDto {
    pub file: String,
    pub line: Option<u32>,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub suggestion: Option<String>,
}

/// Laporan review untuk kartu di pulau dan jendela rincian.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct ReviewDto {
    pub summary: String,
    pub findings: Vec<FindingDto>,
    pub patch_ready: bool,
}

impl From<&ReviewReport> for ReviewDto {
    fn from(r: &ReviewReport) -> Self {
        Self {
            summary: r.summary.clone(),
            findings: r
                .findings
                .iter()
                .map(|f| FindingDto {
                    file: f.file.clone(),
                    line: f.line,
                    severity: f.severity,
                    title: f.title.clone(),
                    detail: f.detail.clone(),
                    suggestion: f.suggestion.clone(),
                })
                .collect(),
            patch_ready: r.patch_ready,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct DayUsageDto {
    pub date: String,
    pub active_min: u32,
    pub avg_brightness: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct HealthSummaryDto {
    pub today_active_min: u32,
    pub brightness: Option<u8>,
    /// Tujuh hari terakhir, terlama dulu.
    pub days: Vec<DayUsageDto>,
}

impl From<&DayUsage> for DayUsageDto {
    fn from(d: &DayUsage) -> Self {
        Self {
            date: d.date.clone(),
            active_min: d.active_min,
            avg_brightness: d.avg_brightness,
        }
    }
}

impl From<&HealthSummary> for HealthSummaryDto {
    fn from(s: &HealthSummary) -> Self {
        Self {
            today_active_min: s.today_active_min,
            brightness: s.brightness,
            days: s.days.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct LyricLineDto {
    pub ms: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
pub struct NetRateDto {
    pub down_bps: u32,
    pub up_bps: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct TaskDto {
    pub id: i32,
    pub title: String,
    pub done: bool,
    pub note: bool,
    pub body: String,
}

impl From<&Task> for TaskDto {
    fn from(t: &Task) -> Self {
        Self {
            id: t.id,
            title: t.title.clone(),
            done: t.done,
            note: t.note,
            body: t.body.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PowerActionDto {
    Shutdown,
    Restart,
    Sleep,
    Lock,
}

impl From<PowerActionDto> for PowerAction {
    fn from(a: PowerActionDto) -> Self {
        match a {
            PowerActionDto::Shutdown => PowerAction::Shutdown,
            PowerActionDto::Restart => PowerAction::Restart,
            PowerActionDto::Sleep => PowerAction::Sleep,
            PowerActionDto::Lock => PowerAction::Lock,
        }
    }
}

/// Dikirim ke semua jendela setiap setelan berubah.
#[derive(Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct SettingsChangedEvent(pub Settings);

/// Dikirim setiap state pulau berubah.
#[derive(Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct IslandSnapshotEvent(pub IslandSnapshot);

impl From<&IslandState> for IslandSnapshot {
    fn from(s: &IslandState) -> Self {
        let mode = if s.fullscreen {
            IslandMode::Hidden
        } else {
            match s.view {
                View::Idle => IslandMode::Idle,
                View::Compact(_) => IslandMode::Compact,
                View::Expanded(_) => IslandMode::Expanded,
            }
        };
        let content = s.view.content().map(|c| match c {
            Content::Media => IslandContent {
                kind: ContentKind::Media,
                text: s
                    .media
                    .as_ref()
                    .map(|m| m.title.clone())
                    .unwrap_or_default(),
                detail: s.media.as_ref().map(|m| m.artist.clone()),
            },
            Content::ReviewRunning(_) => IslandContent {
                kind: ContentKind::ReviewRunning,
                text: String::new(),
                detail: None,
            },
            Content::ReviewResult(r) => IslandContent {
                kind: ContentKind::ReviewResult,
                text: r.summary.clone(),
                detail: None,
            },
            Content::HealthAlert(a) => IslandContent {
                kind: ContentKind::HealthAlert,
                text: a.message.clone(),
                detail: None,
            },
            Content::Volume(v) => IslandContent {
                kind: ContentKind::Volume,
                text: v.level.to_string(),
                detail: Some(if v.muted { "muted" } else { "volume" }.into()),
            },
            Content::Brightness(b) => IslandContent {
                kind: ContentKind::Brightness,
                text: b.to_string(),
                detail: None,
            },
            Content::Ask(p) => {
                let (phase, text) = match p {
                    AskPhase::Input => ("input", String::new()),
                    AskPhase::Thinking => ("thinking", String::new()),
                    AskPhase::Done(t) => ("done", t.clone()),
                    AskPhase::Failed(t) => ("failed", t.clone()),
                };
                IslandContent {
                    kind: ContentKind::Ask,
                    text,
                    detail: Some(phase.into()),
                }
            }
            Content::Shelf => IslandContent {
                kind: ContentKind::Shelf,
                text: s.shelf.len().to_string(),
                detail: None,
            },
            Content::Notice(t) => IslandContent {
                kind: ContentKind::Notice,
                text: t.clone(),
                detail: None,
            },
        });
        let media = match (s.view.content(), &s.media) {
            (Some(Content::Media), Some(m)) => Some(MediaDto {
                playing: m.playing,
                position_ms: m.position_ms,
                duration_ms: m.duration_ms,
                rate: m.rate,
                thumbnail: m.thumbnail.clone(),
                color: m.color.clone(),
                session_index: m.session_index,
                session_count: m.session_count,
            }),
            _ => None,
        };
        let review = match s.view.content() {
            Some(Content::ReviewResult(r)) => Some(r.into()),
            _ => None,
        };
        Self {
            mode,
            content,
            media,
            has_media: s.media.is_some(),
            review,
            mood: mood_for(s),
            extension_connected: s.extension,
            hovered: s.hovered,
            dnd: s.dnd,
            shelf: match s.view.content() {
                Some(Content::Shelf) => s
                    .shelf
                    .iter()
                    .map(|p| ShelfItemDto {
                        path: p.clone(),
                        name: Path::new(p)
                            .file_name()
                            .map_or_else(|| p.clone(), |n| n.to_string_lossy().into_owned()),
                    })
                    .collect(),
                _ => Vec::new(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct StartupAppDto {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub delay_secs: u32,
}

impl From<&StartupApp> for StartupAppDto {
    fn from(a: &StartupApp) -> Self {
        Self {
            id: a.id,
            name: a.name.clone(),
            path: a.path.clone(),
            delay_secs: a.delay_secs,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct InstalledAppDto {
    pub name: String,
    pub path: String,
}

impl From<&InstalledApp> for InstalledAppDto {
    fn from(a: &InstalledApp) -> Self {
        Self {
            name: a.name.clone(),
            path: a.path.clone(),
        }
    }
}

/// Ukuran dikirim sebagai f64 (tepat sampai 9 PB) karena TypeScript tidak punya u64.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct DiskEntryDto {
    pub name: String,
    pub path: String,
    pub size: f64,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct DiskStatusDto {
    pub running: bool,
    /// Folder yang sedang/terakhir dipindai.
    pub path: String,
    /// Folder induknya, kosong bila sudah di akar drive.
    pub parent: Option<String>,
    pub files_scanned: f64,
    /// Ada hasil untuk `path`.
    pub done: bool,
    pub total: f64,
    pub entries: Vec<DiskEntryDto>,
    pub error: Option<String>,
}

impl From<&DiskStatus> for DiskStatusDto {
    fn from(s: &DiskStatus) -> Self {
        let report = s.report.as_ref();
        Self {
            running: s.running,
            path: s.path.clone(),
            parent: Path::new(&s.path)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .filter(|p| !p.is_empty()),
            files_scanned: s.files_scanned as f64,
            done: report.is_some(),
            total: report.map_or(0.0, |r| r.total as f64),
            entries: report.map_or_else(Vec::new, |r| {
                r.entries
                    .iter()
                    .map(|e| DiskEntryDto {
                        name: e.name.clone(),
                        path: e.path.clone(),
                        size: e.size as f64,
                        is_dir: e.is_dir,
                    })
                    .collect()
            }),
            error: s.error.clone(),
        }
    }
}
