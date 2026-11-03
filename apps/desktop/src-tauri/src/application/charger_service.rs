use std::sync::{Arc, Mutex};

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::island::action::IslandAction;
use crate::domain::model::settings::Language;
use crate::domain::ports::PowerReader;

/// Munculkan notifikasi di pulau saat charger dicolok.
pub struct ChargerService {
    power: Arc<dyn PowerReader>,
    settings: Arc<SettingsService>,
    island: Arc<IslandService>,
    /// Status colokan pada pembacaan sebelumnya; `None` sebelum pembacaan pertama.
    last: Mutex<Option<bool>>,
}

impl ChargerService {
    pub fn new(
        power: Arc<dyn PowerReader>,
        settings: Arc<SettingsService>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            power,
            settings,
            island,
            last: Mutex::new(None),
        })
    }

    /// Dipanggil berkala. Pembacaan pertama hanya menjadi acuan, jadi tidak ada notifikasi saat start.
    pub fn tick(&self) {
        let Some(status) = self.power.status() else {
            return;
        };
        let was = self
            .last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(status.plugged);
        if was != Some(false) || !status.plugged {
            return;
        }
        let pct = status
            .percent
            .map(|p| format!(" · {p}%"))
            .unwrap_or_default();
        let text = match self.settings.get().language {
            Language::Id => format!("⚡ Mengisi daya{pct}"),
            Language::En => format!("⚡ Charging{pct}"),
        };
        self.island.dispatch(IslandAction::Notice(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::settings::Settings;
    use crate::domain::ports::PowerStatus;

    struct Fake(Mutex<Option<PowerStatus>>);
    impl PowerReader for Fake {
        fn status(&self) -> Option<PowerStatus> {
            *self.0.lock().unwrap()
        }
    }

    fn set(f: &Fake, plugged: bool, percent: Option<u8>) {
        *f.0.lock().unwrap() = Some(PowerStatus { plugged, percent });
    }

    #[test]
    fn notifikasi_hanya_saat_berubah_dari_dicabut_ke_dicolok() {
        let (settings, island, _rt) = services(Settings::default());
        let power = Arc::new(Fake(Mutex::new(Some(PowerStatus {
            plugged: true,
            percent: Some(50),
        }))));
        let svc = ChargerService::new(power.clone(), settings, island.clone());

        svc.tick(); // acuan awal sudah dicolok: diam
        assert_eq!(island.snapshot().view, View::Idle);

        set(&power, false, Some(49));
        svc.tick();
        assert_eq!(island.snapshot().view, View::Idle);

        set(&power, true, Some(80));
        svc.tick();
        assert!(matches!(
            island.snapshot().view,
            View::Expanded(Content::Notice(t)) if t.contains("80%")
        ));
    }

    #[test]
    fn tanpa_baterai_tidak_ada_notifikasi() {
        let (settings, island, _rt) = services(Settings::default());
        let svc = ChargerService::new(Arc::new(Fake(Mutex::new(None))), settings, island.clone());
        svc.tick();
        svc.tick();
        assert_eq!(island.snapshot().view, View::Idle);
    }
}
