use std::sync::{Arc, Mutex};

use crate::application::island_service::IslandService;
use crate::domain::island::action::IslandAction;
use crate::domain::ports::BrightnessReader;

/// Munculkan HUD kecerahan di pulau saat kecerahan layar internal berubah.
pub struct BrightnessService {
    reader: Arc<dyn BrightnessReader>,
    island: Arc<IslandService>,
    /// Pembacaan sebelumnya; `None` sebelum pembacaan pertama.
    last: Mutex<Option<u8>>,
}

impl BrightnessService {
    pub fn new(reader: Arc<dyn BrightnessReader>, island: Arc<IslandService>) -> Arc<Self> {
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
            self.island.dispatch(IslandAction::BrightnessChanged(now));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};

    struct Fake(Mutex<Option<u8>>);
    impl BrightnessReader for Fake {
        fn read(&self) -> Option<u8> {
            *self.0.lock().unwrap()
        }
    }

    #[test]
    fn hud_hanya_saat_berubah_bukan_saat_start_atau_tetap() {
        let (_settings, island, _rt) = services(Default::default());
        let fake = Arc::new(Fake(Mutex::new(None)));
        let svc = BrightnessService::new(fake.clone(), island.clone());
        svc.tick(); // layar tanpa kontrol kecerahan
        *fake.0.lock().unwrap() = Some(50);
        svc.tick(); // acuan awal
        svc.tick(); // tidak berubah
        assert_eq!(island.snapshot().view, View::Idle);
        *fake.0.lock().unwrap() = Some(60);
        svc.tick();
        assert_eq!(
            island.snapshot().view,
            View::Compact(Content::Brightness(60))
        );
    }
}
