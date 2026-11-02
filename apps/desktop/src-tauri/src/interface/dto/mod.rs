//! DTO yang dikirim ke frontend. Dihasilkan dari state domain; tipe TS dibuat otomatis.
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::application::health_service::HealthSummary;
use crate::domain::health::DayUsage;
use crate::domain::island::state::{Content, IslandState, View};
use crate::domain::mascot::{Mood, mood_for};
use crate::domain::model::review::Severity;
use crate::domain::model::settings::Settings;
use crate::domain::model::{MediaCommand, ReviewReport};

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
    /// Laporan lengkap bila pulau menampilkan hasil review.
    pub review: Option<ReviewDto>,
    /// Ekspresi Lumi untuk keadaan pulau saat ini.
    pub mood: Mood,
    pub hovered: bool,
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
            review,
            mood: mood_for(s),
            hovered: s.hovered,
        }
    }
}
