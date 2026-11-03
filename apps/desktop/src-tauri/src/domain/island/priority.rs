use super::state::Content;

impl Content {
    /// Makin besar makin penting: Ask (dipanggil pengguna) > ReviewResult > ReviewRunning > HealthAlert > Notice > Media.
    pub fn priority(&self) -> u8 {
        match self {
            Content::Ask(_) => 6,
            Content::ReviewResult(_) | Content::Shelf => 5,
            Content::ReviewRunning(_) => 4,
            Content::HealthAlert(_) => 3,
            Content::Notice(_) | Content::Volume(_) | Content::Brightness(_) => 2,
            Content::Media => 1,
        }
    }
}
