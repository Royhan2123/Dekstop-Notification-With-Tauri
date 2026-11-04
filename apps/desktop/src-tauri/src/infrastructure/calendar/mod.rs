use crate::domain::error::CalendarError;
use crate::domain::ports::CalendarSource;
use crate::infrastructure::theme::https_get;

/// Kalender besar (setahun lebih, banyak acara berulang) bisa lebih dari 1 MB.
const MAX_CALENDAR_BYTES: usize = 5_000_000;

/// Mengunduh kalender iCal lewat HTTPS, mis. alamat rahasia format iCal dari Google Calendar.
pub struct HttpCalendarFeed;

impl CalendarSource for HttpCalendarFeed {
    fn fetch(&self, url: &str) -> Result<String, CalendarError> {
        https_get(url, MAX_CALENDAR_BYTES).map_err(|e| CalendarError::Unavailable(e.to_string()))
    }
}
