use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::application::settings_service::SettingsService;
use crate::domain::error::LyricsError;
use crate::domain::lyrics::{LyricLine, parse_lrc, pick_best, search_queries};
use crate::domain::ports::LyricsSource;

/// Cache dalam memori; penuh berarti dikosongkan (pemutaran biasanya berulang di beberapa lagu saja).
const CACHE_MAX: usize = 60;

/// Mencari lirik berwaktu untuk lagu yang sedang diputar.
pub struct LyricsService {
    source: Arc<dyn LyricsSource>,
    settings: Arc<SettingsService>,
    cache: Mutex<HashMap<String, Vec<LyricLine>>>,
}

impl LyricsService {
    pub fn new(source: Arc<dyn LyricsSource>, settings: Arc<SettingsService>) -> Arc<Self> {
        Arc::new(Self {
            source,
            settings,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Daftar kosong berarti tidak ada lirik (belum tentu galat). Galat jaringan tidak disimpan di cache
    /// supaya dicoba lagi nanti; "tidak ada" disimpan supaya server tidak ditanya berulang.
    pub fn fetch(
        &self,
        artist: &str,
        title: &str,
        duration_ms: u32,
    ) -> Result<Vec<LyricLine>, LyricsError> {
        // Tanpa durasi, hasil pencarian tidak bisa dipastikan lagu yang sama.
        if !self.settings.get().lyrics_enabled || duration_ms == 0 {
            return Ok(Vec::new());
        }
        let key = format!("{artist}\u{1}{title}\u{1}{}", duration_ms / 1000);
        if let Some(hit) = self.cache().get(&key) {
            return Ok(hit.clone());
        }
        let mut failure = None;
        for query in search_queries(artist, title) {
            match self.source.search(&query) {
                Ok(found) => {
                    let lines = pick_best(&found, artist, title, duration_ms)
                        .and_then(|c| c.synced.as_deref())
                        .map(parse_lrc)
                        .unwrap_or_default();
                    if !lines.is_empty() {
                        self.store(key, lines.clone());
                        return Ok(lines);
                    }
                }
                Err(e) => failure = Some(e),
            }
        }
        if let Some(e) = failure {
            return Err(e);
        }
        self.store(key, Vec::new());
        Ok(Vec::new())
    }

    fn store(&self, key: String, lines: Vec<LyricLine>) {
        let mut cache = self.cache();
        if cache.len() >= CACHE_MAX {
            cache.clear();
        }
        cache.insert(key, lines);
    }

    fn cache(&self) -> std::sync::MutexGuard<'_, HashMap<String, Vec<LyricLine>>> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::lyrics::Candidate;
    use crate::domain::model::settings::Settings;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fake {
        calls: AtomicUsize,
        reply: Mutex<Result<Vec<Candidate>, ()>>,
    }
    impl Fake {
        fn new(reply: Result<Vec<Candidate>, ()>) -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                reply: Mutex::new(reply),
            })
        }
    }
    impl LyricsSource for Fake {
        fn search(&self, _query: &str) -> Result<Vec<Candidate>, LyricsError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.reply
                .lock()
                .unwrap()
                .clone()
                .map_err(|_| LyricsError::Unavailable("sibuk".into()))
        }
    }

    fn hit() -> Candidate {
        Candidate {
            artist: "Artis".into(),
            track: "Judul".into(),
            duration_secs: 200.0,
            synced: Some("[00:01.00]halo\n[00:05.00]dunia".into()),
        }
    }

    fn svc(fake: &Arc<Fake>, settings: Settings) -> (Arc<LyricsService>, tokio::runtime::Runtime) {
        let (settings, _island, rt) = services(settings);
        (LyricsService::new(fake.clone(), settings), rt)
    }

    #[test]
    fn menemukan_mengurai_dan_menyimpan_di_cache() {
        let fake = Fake::new(Ok(vec![hit()]));
        let (svc, _rt) = svc(&fake, Settings::default());
        let lines = svc.fetch("Artis", "Judul", 200_000).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "halo");
        assert_eq!(svc.fetch("Artis", "Judul", 200_500).unwrap(), lines);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1, "kedua dari cache");
    }

    #[test]
    fn tidak_ada_disimpan_tetapi_galat_tidak() {
        let fake = Fake::new(Ok(vec![]));
        let (svc, _rt) = svc(&fake, Settings::default());
        assert!(svc.fetch("A", "B", 100_000).unwrap().is_empty());
        let calls = fake.calls.load(Ordering::SeqCst);
        assert!(svc.fetch("A", "B", 100_000).unwrap().is_empty());
        assert_eq!(
            fake.calls.load(Ordering::SeqCst),
            calls,
            "tidak bertanya lagi"
        );

        let busy = Fake::new(Err(()));
        let (svc, _rt) = svc_busy(&busy);
        assert!(svc.fetch("A", "B", 100_000).is_err());
        assert!(svc.fetch("A", "B", 100_000).is_err());
        assert!(busy.calls.load(Ordering::SeqCst) >= 2, "galat dicoba lagi");
        *busy.reply.lock().unwrap() = Ok(vec![hit()]);
        assert_eq!(svc.fetch("Artis", "Judul", 200_000).unwrap().len(), 2);
    }

    fn svc_busy(fake: &Arc<Fake>) -> (Arc<LyricsService>, tokio::runtime::Runtime) {
        svc(fake, Settings::default())
    }

    #[test]
    fn dimatikan_atau_tanpa_durasi_tidak_menghubungi_server() {
        let fake = Fake::new(Ok(vec![hit()]));
        let (off, _rt) = svc(
            &fake,
            Settings {
                lyrics_enabled: false,
                ..Default::default()
            },
        );
        assert!(off.fetch("A", "B", 200_000).unwrap().is_empty());
        let (on, _rt2) = svc(&fake, Settings::default());
        assert!(on.fetch("A", "B", 0).unwrap().is_empty());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn durasi_yang_tidak_cocok_ditolak() {
        let fake = Fake::new(Ok(vec![hit()]));
        let (svc, _rt) = svc(&fake, Settings::default());
        assert!(svc.fetch("Artis", "Judul", 350_000).unwrap().is_empty());
    }
}
