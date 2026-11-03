use std::sync::{Arc, Mutex};

use crate::application::island_service::IslandService;
use crate::domain::island::action::IslandAction;
use crate::domain::island::state::VolumeInfo;
use crate::domain::ports::VolumeReader;

/// Munculkan HUD volume di pulau saat volume atau bisu berubah.
pub struct VolumeService {
    reader: Arc<dyn VolumeReader>,
    island: Arc<IslandService>,
    /// Pembacaan sebelumnya; `None` sebelum pembacaan pertama.
    last: Mutex<Option<VolumeInfo>>,
}

impl VolumeService {
    pub fn new(reader: Arc<dyn VolumeReader>, island: Arc<IslandService>) -> Arc<Self> {
        Arc::new(Self {
            reader,
            island,
            last: Mutex::new(None),
        })
    }

    /// Dipanggil berkala. Pembacaan pertama hanya menjadi acuan, jadi tidak ada HUD saat start.
    pub fn tick(&self) {
        let Some(now) = self.reader.read() else {
            return;
        };
        let was = self
            .last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(now);
        if was.is_some_and(|w| w != now) {
            self.island.dispatch(IslandAction::VolumeChanged(now));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};

    struct Fake(Mutex<Option<VolumeInfo>>);
    impl VolumeReader for Fake {
        fn read(&self) -> Option<VolumeInfo> {
            *self.0.lock().unwrap()
        }
    }

    fn set(f: &Fake, level: u8, muted: bool) {
        *f.0.lock().unwrap() = Some(VolumeInfo { level, muted });
    }

    #[test]
    fn hud_hanya_saat_berubah_bukan_saat_start_atau_tetap() {
        let (_settings, island, _rt) = services(Default::default());
        let fake = Arc::new(Fake(Mutex::new(None)));
        let svc = VolumeService::new(fake.clone(), island.clone());

        svc.tick(); // perangkat belum terbaca
        set(&fake, 30, false);
        svc.tick(); // acuan awal
        svc.tick(); // tidak berubah
        assert_eq!(island.snapshot().view, View::Idle);

        set(&fake, 32, false);
        svc.tick();
        assert!(matches!(
            island.snapshot().view,
            View::Compact(Content::Volume(v)) if v.level == 32 && !v.muted
        ));

        set(&fake, 32, true);
        svc.tick();
        assert!(matches!(
            island.snapshot().view,
            View::Compact(Content::Volume(v)) if v.muted
        ));
    }
}
