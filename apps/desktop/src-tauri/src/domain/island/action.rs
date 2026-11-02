use crate::domain::model::{HealthAlert, MediaInfo, ReviewId, ReviewReport};

pub enum IslandAction {
    MediaChanged(Option<MediaInfo>),
    ReviewStarted(ReviewId),
    ReviewFinished(ReviewId, ReviewReport),
    HealthAlert(HealthAlert),
    Notice(String),
    Hover(bool),
    Clicked,
    Dismissed,
    /// Timer dengan token tertentu habis; diabaikan bila token sudah usang.
    Timeout(u64),
    HideForFullscreen(bool),
    DoNotDisturb(bool),
    ExtensionConnected(bool),
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    StartTimer { token: u64, ms: u64 },
}
