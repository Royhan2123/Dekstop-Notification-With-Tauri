//! Model domain. Field akan bertambah seiring fitur (media, review, kesehatan).

#[derive(Clone, Debug, PartialEq)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewReport {
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthAlert {
    pub message: String,
}
