use std::sync::{Arc, Mutex, MutexGuard};

use crate::application::island_service::IslandService;
use crate::domain::error::MediaError;
use crate::domain::island::action::IslandAction;
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::ports::MediaSource;

#[derive(Default)]
struct Inner {
    sessions: Vec<MediaInfo>,
    /// Sesi yang dipilih pengguna (scroll); `None` = otomatis (yang sedang diputar).
    selected: Option<String>,
}

/// Memilih sesi media yang tampil di pulau dan meneruskan kontrol ke OS.
pub struct MediaService {
    source: Arc<dyn MediaSource>,
    island: Arc<IslandService>,
    inner: Mutex<Inner>,
}

impl MediaService {
    pub fn new(source: Arc<dyn MediaSource>, island: Arc<IslandService>) -> Arc<Self> {
        Arc::new(Self {
            source,
            island,
            inner: Mutex::new(Inner::default()),
        })
    }

    pub fn start(self: &Arc<Self>) {
        let this = Arc::clone(self);
        self.source.start(Box::new(move |sessions| {
            this.lock().sessions = sessions;
            this.publish();
        }));
    }

    pub fn control(&self, cmd: MediaCommand) -> Result<(), MediaError> {
        let id = {
            let inner = self.lock();
            let i =
                pick(&inner.sessions, inner.selected.as_deref()).ok_or(MediaError::NoSession)?;
            inner.sessions[i].session_id.clone()
        };
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

    fn publish(&self) {
        let media = {
            let inner = self.lock();
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
}
