use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::application::island_service::IslandService;
use crate::domain::bridge::{BridgeCommand, BrowserMedia, session_id, tab_of_session};
use crate::domain::error::MediaError;
use crate::domain::island::action::IslandAction;
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::ports::{BrowserBridge, MediaSource};

/// Gambar sampul dari URL: (data URL, warna dominan). Diinjeksi agar layer ini bebas dari HTTP.
pub type ArtFetcher = Arc<dyn Fn(&str) -> Option<(String, String)> + Send + Sync>;

/// Sampul per URL; `None` = sudah dicoba dan gagal (jangan diulang tiap pembaruan).
type ArtCache = HashMap<String, Option<(String, String)>>;
const ART_CACHE_MAX: usize = 64;

#[derive(Default)]
struct Inner {
    /// Sesi dari OS (Music, Spotify, SMTC).
    system: Vec<MediaInfo>,
    /// Sesi dari tab browser lewat ekstensi Isla Link.
    browser: Vec<BrowserMedia>,
    /// Gabungan keduanya (sistem dulu), yang dipilih untuk tampil.
    sessions: Vec<MediaInfo>,
    /// Sesi yang dipilih pengguna (scroll); `None` = otomatis (yang sedang diputar).
    selected: Option<String>,
}

/// Memilih sesi media yang tampil di pulau dan meneruskan kontrol ke OS.
pub struct MediaService {
    source: Arc<dyn MediaSource>,
    island: Arc<IslandService>,
    inner: Mutex<Inner>,
    bridge: Mutex<Option<Arc<dyn BrowserBridge>>>,
    art_fetcher: Mutex<Option<ArtFetcher>>,
    art: Mutex<ArtCache>,
}

impl MediaService {
    pub fn new(source: Arc<dyn MediaSource>, island: Arc<IslandService>) -> Arc<Self> {
        Arc::new(Self {
            source,
            island,
            inner: Mutex::new(Inner::default()),
            bridge: Mutex::new(None),
            art_fetcher: Mutex::new(None),
            art: Mutex::new(HashMap::new()),
        })
    }

    /// Hubungkan sesi browser: `bridge` untuk mengirim kontrol ke tab, `fetch` untuk sampul.
    pub fn attach_browser(&self, bridge: Arc<dyn BrowserBridge>, fetch: ArtFetcher) {
        *self.bridge.lock().unwrap_or_else(|e| e.into_inner()) = Some(bridge);
        *self.art_fetcher.lock().unwrap_or_else(|e| e.into_inner()) = Some(fetch);
    }

    pub fn start(self: &Arc<Self>) {
        let this = Arc::clone(self);
        self.source.start(Box::new(move |sessions| {
            this.lock().system = sessions;
            this.publish();
        }));
    }

    /// Daftar tab browser terbaru dari ekstensi (kosong = tidak ada / sambungan putus).
    pub fn set_browser(self: &Arc<Self>, sessions: Vec<BrowserMedia>) {
        let missing: Vec<String> = {
            let cache = self.art_cache();
            sessions
                .iter()
                .filter_map(|s| s.artwork.clone())
                .filter(|u| !cache.contains_key(u))
                .collect()
        };
        self.lock().browser = sessions;
        self.publish();
        // Sampul diunduh di thread sendiri supaya jalur WebSocket tidak tertahan.
        let fetcher = self
            .art_fetcher
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let (Some(fetch), false) = (fetcher, missing.is_empty()) {
            let this = Arc::clone(self);
            std::thread::spawn(move || {
                for url in missing {
                    let got = fetch(&url);
                    let mut cache = this.art_cache();
                    if cache.len() >= ART_CACHE_MAX {
                        cache.clear();
                    }
                    cache.insert(url, got);
                }
                this.publish();
            });
        }
    }

    pub fn control(&self, cmd: MediaCommand) -> Result<(), MediaError> {
        let id = {
            let inner = self.lock();
            let i =
                pick(&inner.sessions, inner.selected.as_deref()).ok_or(MediaError::NoSession)?;
            inner.sessions[i].session_id.clone()
        };
        if let Some(tab_id) = tab_of_session(&id) {
            let bridge = self
                .bridge
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
                .ok_or(MediaError::NoSession)?;
            return bridge
                .send(&BridgeCommand::MediaControl {
                    tab_id,
                    action: cmd,
                })
                .map_err(|e| MediaError::Failed(e.to_string()));
        }
        self.source.control(&id, cmd)
    }

    /// Pindah ke sesi berikutnya (`delta` > 0) atau sebelumnya, berputar.
    pub fn cycle(&self, delta: i32) {
        {
            let mut inner = self.lock();
            let Some(i) = pick(&inner.sessions, inner.selected.as_deref()) else {
                return;
            };
            let len = inner.sessions.len() as i32;
            let next = (i as i32 + delta).rem_euclid(len) as usize;
            inner.selected = Some(inner.sessions[next].session_id.clone());
        }
        self.publish();
    }

    /// Gabungkan sesi sistem dan browser, lalu kirim yang terpilih ke pulau.
    fn publish(&self) {
        let browser: Vec<MediaInfo> = {
            let cache = self.art_cache();
            self.lock()
                .browser
                .iter()
                .map(|b| {
                    let art = b.artwork.as_ref().and_then(|u| cache.get(u)).cloned();
                    let art = art.flatten();
                    MediaInfo {
                        session_id: session_id(b.tab_id),
                        title: b.title.clone(),
                        artist: b.artist.clone(),
                        playing: b.playing,
                        position_ms: b.position_ms,
                        duration_ms: b.duration_ms,
                        rate: b.rate,
                        thumbnail: art.as_ref().map(|a| a.0.clone()),
                        color: art.map(|a| a.1),
                        ..Default::default()
                    }
                })
                .collect()
        };
        let media = {
            let mut inner = self.lock();
            inner.sessions = inner.system.iter().cloned().chain(browser).collect();
            let inner = inner;
            pick(&inner.sessions, inner.selected.as_deref()).map(|i| MediaInfo {
                session_index: i as u32,
                session_count: inner.sessions.len() as u32,
                ..inner.sessions[i].clone()
            })
        };
        self.island.dispatch(IslandAction::MediaChanged(media));
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn art_cache(&self) -> MutexGuard<'_, ArtCache> {
        self.art.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Urutan pilihan: sesi pilihan pengguna, lalu yang sedang diputar, lalu yang pertama.
fn pick(sessions: &[MediaInfo], selected: Option<&str>) -> Option<usize> {
    selected
        .and_then(|id| sessions.iter().position(|s| s.session_id == id))
        .or_else(|| sessions.iter().position(|s| s.playing))
        .or(if sessions.is_empty() { None } else { Some(0) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::island::state::IslandState;
    use crate::domain::ports::SnapshotSink;
    use tokio::runtime::Handle;

    fn s(id: &str, playing: bool) -> MediaInfo {
        MediaInfo {
            session_id: id.into(),
            title: id.into(),
            playing,
            ..Default::default()
        }
    }

    #[test]
    fn pick_memilih_pilihan_pengguna_lalu_yang_diputar_lalu_pertama() {
        let list = [s("a", false), s("b", true), s("c", false)];
        assert_eq!(pick(&list, Some("c")), Some(2));
        assert_eq!(pick(&list, Some("hilang")), Some(1));
        assert_eq!(pick(&list, None), Some(1));
        assert_eq!(pick(&[s("a", false), s("b", false)], None), Some(0));
        assert_eq!(pick(&[], None), None);
    }

    struct NullSink;
    impl SnapshotSink for NullSink {
        fn publish(&self, _: &IslandState) {}
    }

    type OnChange = Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>;

    #[derive(Default)]
    struct FakeSource {
        on_change: Mutex<Option<OnChange>>,
        controlled: Mutex<Vec<(String, MediaCommand)>>,
    }
    impl MediaSource for Arc<FakeSource> {
        fn start(&self, f: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
            *self.on_change.lock().unwrap() = Some(f);
        }
        fn control(&self, id: &str, cmd: MediaCommand) -> Result<(), MediaError> {
            self.controlled.lock().unwrap().push((id.into(), cmd));
            Ok(())
        }
    }

    #[tokio::test]
    async fn sesi_dari_source_sampai_ke_pulau_dan_kontrol_ke_sesi_terpilih() {
        let source = Arc::new(FakeSource::default());
        let island = IslandService::new(Arc::new(NullSink), Handle::current());
        let svc = MediaService::new(Arc::new(source.clone()), island.clone());
        assert!(matches!(
            svc.control(MediaCommand::Next),
            Err(MediaError::NoSession)
        ));

        svc.start();
        let emit = |v| (source.on_change.lock().unwrap().as_ref().unwrap())(v);
        emit(vec![s("a", false), s("b", true)]);
        let media = island.snapshot().media.unwrap();
        assert_eq!(
            (
                media.title.as_str(),
                media.session_index,
                media.session_count
            ),
            ("b", 1, 2)
        );

        svc.cycle(1); // berputar ke "a"
        assert_eq!(island.snapshot().media.unwrap().title, "a");
        svc.control(MediaCommand::TogglePlay).unwrap();
        assert_eq!(
            source.controlled.lock().unwrap()[0],
            ("a".to_string(), MediaCommand::TogglePlay)
        );

        emit(vec![]);
        assert!(island.snapshot().media.is_none());
    }

    #[derive(Default)]
    struct FakeBridge {
        sent: Mutex<Vec<BridgeCommand>>,
    }
    impl BrowserBridge for FakeBridge {
        fn send(&self, cmd: &BridgeCommand) -> Result<(), crate::domain::error::BridgeError> {
            self.sent.lock().unwrap().push(cmd.clone());
            Ok(())
        }
        fn disconnect(&self) {}
    }

    fn tab(id: u64, title: &str, playing: bool) -> BrowserMedia {
        BrowserMedia {
            tab_id: id,
            title: title.into(),
            artist: "Channel".into(),
            playing,
            position_ms: 1000,
            duration_ms: 200_000,
            rate: 1.0,
            artwork: None,
        }
    }

    #[tokio::test]
    async fn sesi_browser_tampil_bergabung_dengan_sistem_dan_dikontrol_lewat_jembatan() {
        let source = Arc::new(FakeSource::default());
        let island = IslandService::new(Arc::new(NullSink), Handle::current());
        let svc = MediaService::new(Arc::new(source.clone()), island.clone());
        let bridge = Arc::new(FakeBridge::default());
        svc.attach_browser(bridge.clone(), Arc::new(|_| None));
        svc.start();

        svc.set_browser(vec![tab(5, "Video YouTube", true)]);
        let media = island.snapshot().media.unwrap();
        assert_eq!(
            (media.title.as_str(), media.session_id.as_str()),
            ("Video YouTube", "browser:5")
        );

        // Sesi sistem yang sedang diam tidak mengalahkan tab yang sedang diputar.
        (source.on_change.lock().unwrap().as_ref().unwrap())(vec![s("Music", false)]);
        let media = island.snapshot().media.unwrap();
        assert_eq!(
            (media.title.as_str(), media.session_count),
            ("Video YouTube", 2)
        );

        svc.control(MediaCommand::TogglePlay).unwrap();
        assert_eq!(
            bridge.sent.lock().unwrap()[0],
            BridgeCommand::MediaControl {
                tab_id: 5,
                action: MediaCommand::TogglePlay
            }
        );
        assert!(source.controlled.lock().unwrap().is_empty(), "tidak ke OS");

        // Sambungan ekstensi putus: tab hilang, tersisa sesi sistem.
        svc.set_browser(Vec::new());
        assert_eq!(island.snapshot().media.unwrap().title, "Music");
    }

    #[tokio::test]
    async fn kontrol_sesi_browser_tanpa_jembatan_gagal_bersih() {
        let source = Arc::new(FakeSource::default());
        let island = IslandService::new(Arc::new(NullSink), Handle::current());
        let svc = MediaService::new(Arc::new(source), island);
        svc.start();
        svc.set_browser(vec![tab(1, "x", true)]);
        assert!(matches!(
            svc.control(MediaCommand::Next),
            Err(MediaError::NoSession)
        ));
    }
}
