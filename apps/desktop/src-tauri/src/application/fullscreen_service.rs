use std::sync::{Arc, Mutex};

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::island::action::IslandAction;
use crate::domain::ports::FullscreenDetector;

/// Menyembunyikan pulau selama ada aplikasi layar penuh, dan menampilkannya lagi sesudahnya.
pub struct FullscreenService {
    detector: Arc<dyn FullscreenDetector>,
    settings: Arc<SettingsService>,
    island: Arc<IslandService>,
    /// Status terakhir yang dikirim ke pulau; hanya perubahan yang dikirim.
    last: Mutex<bool>,
}

impl FullscreenService {
    pub fn new(
        detector: Arc<dyn FullscreenDetector>,
        settings: Arc<SettingsService>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            detector,
            settings,
            island,
            last: Mutex::new(false),
        })
    }

    /// Dipanggil berkala. Setelan dibaca tiap putaran, jadi mematikannya langsung memunculkan pulau lagi.
    pub fn tick(&self) {
        let hide = self.settings.get().hide_on_fullscreen && self.detector.is_fullscreen();
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if *last != hide {
            *last = hide;
            self.island.dispatch(IslandAction::HideForFullscreen(hide));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::model::settings::Settings;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct Fake(AtomicBool);
    impl FullscreenDetector for Fake {
        fn is_fullscreen(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn pulau_disembunyikan_selama_layar_penuh_lalu_muncul_lagi() {
        let (settings, island, _rt) = services(Settings::default());
        let fake = Arc::new(Fake(AtomicBool::new(false)));
        let svc = FullscreenService::new(fake.clone(), settings, island.clone());
        svc.tick();
        assert!(!island.snapshot().fullscreen);
        fake.0.store(true, Ordering::SeqCst);
        svc.tick();
        assert!(island.snapshot().fullscreen);
        fake.0.store(false, Ordering::SeqCst);
        svc.tick();
        assert!(!island.snapshot().fullscreen);
    }

    #[test]
    fn setelan_mati_membuat_pulau_tetap_tampil() {
        let (settings, island, _rt) = services(Settings {
            hide_on_fullscreen: false,
            ..Default::default()
        });
        let svc = FullscreenService::new(
            Arc::new(Fake(AtomicBool::new(true))),
            settings,
            island.clone(),
        );
        svc.tick();
        assert!(!island.snapshot().fullscreen);
    }
}
