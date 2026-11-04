use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::NaiveDateTime;

use crate::application::settings_service::SettingsService;
use crate::domain::calendar::{Meeting, next_meeting};
use crate::domain::error::CalendarError;
use crate::domain::ports::CalendarSource;

/// Berkas kalender diunduh ulang paling sering sekali dalam selang ini.
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);
/// Pengingat muncul saat rapat tinggal sejauh ini.
const REMIND_BEFORE_MINUTES: i64 = 5;

struct Cached {
    url: String,
    at: Instant,
    ics: String,
}

/// Rapat berikutnya dari tautan iCal di setelan, plus pengingat sebelum mulai.
pub struct CalendarService {
    source: Arc<dyn CalendarSource>,
    settings: Arc<SettingsService>,
    cache: Mutex<Option<Cached>>,
    reminded: Mutex<HashSet<NaiveDateTime>>,
}

impl CalendarService {
    pub fn new(source: Arc<dyn CalendarSource>, settings: Arc<SettingsService>) -> Arc<Self> {
        Arc::new(Self {
            source,
            settings,
            cache: Mutex::new(None),
            reminded: Mutex::new(HashSet::new()),
        })
    }

    /// `Ok(None)` bila tautan kalender belum diisi atau tidak ada rapat dalam dua minggu ke depan.
    /// `now` dan `offset_secs`: jam dinding komputer dan selisihnya dari UTC.
    pub fn next(
        &self,
        now: NaiveDateTime,
        offset_secs: i32,
    ) -> Result<Option<Meeting>, CalendarError> {
        let url = self.settings.get().calendar_url;
        if url.is_empty() {
            return Ok(None);
        }
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        let stale = cache
            .as_ref()
            .is_none_or(|c| c.url != url || c.at.elapsed() > CACHE_TTL);
        if stale {
            let ics = self.source.fetch(&url)?;
            *cache = Some(Cached {
                url,
                at: Instant::now(),
                ics,
            });
        }
        Ok(cache
            .as_ref()
            .and_then(|c| next_meeting(&c.ics, now, offset_secs)))
    }

    /// Kalimat pengingat bila ada rapat yang mulai dalam lima menit dan belum diingatkan.
    pub fn due_reminder(&self, now: NaiveDateTime, offset_secs: i32) -> Option<String> {
        let meeting = self.next(now, offset_secs).ok().flatten()?;
        let minutes = (meeting.start - now).num_minutes();
        if minutes > REMIND_BEFORE_MINUTES {
            return None;
        }
        let mut reminded = self.reminded.lock().unwrap_or_else(|e| e.into_inner());
        if !reminded.insert(meeting.start) {
            return None;
        }
        // Hanya rapat yang masih akan datang yang diingat; daftar lama dibuang agar tidak menumpuk.
        reminded.retain(|t| *t >= now);
        Some(match minutes {
            0 => format!("Rapat \"{}\" dimulai sekarang", meeting.title),
            m => format!("Rapat \"{}\" mulai {m} menit lagi", meeting.title),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::model::settings::Settings;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Feed(AtomicUsize, String);
    impl CalendarSource for Feed {
        fn fetch(&self, _: &str) -> Result<String, CalendarError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(self.1.clone())
        }
    }

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    fn make(url: &str) -> (Arc<CalendarService>, Arc<Feed>, tokio::runtime::Runtime) {
        let (settings, _island, rt) = services(Settings {
            calendar_url: url.into(),
            ..Default::default()
        });
        let ics = "BEGIN:VEVENT\nUID:a\nSUMMARY:Demo\nDTSTART:20261103T100000\nEND:VEVENT\n";
        let feed = Arc::new(Feed(AtomicUsize::new(0), ics.into()));
        (CalendarService::new(feed.clone(), settings), feed, rt)
    }

    #[test]
    fn tanpa_tautan_tidak_menghubungi_jaringan() {
        let (svc, feed, _rt) = make("");
        assert_eq!(svc.next(at("2026-11-03 09:00"), 0).unwrap(), None);
        assert_eq!(feed.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn berkas_disimpan_sementara_dan_rapat_ditemukan() {
        let (svc, feed, _rt) = make("https://example.com/c.ics");
        let m = svc.next(at("2026-11-03 09:00"), 0).unwrap().unwrap();
        assert_eq!(m.title, "Demo");
        svc.next(at("2026-11-03 09:01"), 0).unwrap();
        assert_eq!(feed.0.load(Ordering::SeqCst), 1, "kedua dari cache");
    }

    #[test]
    fn pengingat_sekali_per_rapat_dan_hanya_menjelang_mulai() {
        let (svc, _feed, _rt) = make("https://example.com/c.ics");
        assert_eq!(
            svc.due_reminder(at("2026-11-03 09:00"), 0),
            None,
            "masih jauh"
        );
        let r = svc.due_reminder(at("2026-11-03 09:56"), 0).unwrap();
        assert_eq!(r, "Rapat \"Demo\" mulai 4 menit lagi");
        assert_eq!(
            svc.due_reminder(at("2026-11-03 09:57"), 0),
            None,
            "sudah diingatkan"
        );
    }
}
