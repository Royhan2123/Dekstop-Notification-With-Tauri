use std::sync::{Arc, Mutex};

use crate::application::settings_service::SettingsService;
use crate::domain::clipboard::{ClipEntry, ClipboardHistory};
use crate::domain::ports::ClipboardAccess;

#[derive(Default)]
struct State {
    history: ClipboardHistory,
    /// Isi clipboard pada pembacaan terakhir, agar isi yang tidak berubah tidak dicatat berulang.
    last: Option<String>,
}

/// Riwayat clipboard di memori saja (tidak pernah ditulis ke disk) dan hanya aktif bila diizinkan di setelan.
pub struct ClipboardService {
    access: Arc<dyn ClipboardAccess>,
    settings: Arc<SettingsService>,
    state: Mutex<State>,
}

impl ClipboardService {
    pub fn new(access: Arc<dyn ClipboardAccess>, settings: Arc<SettingsService>) -> Arc<Self> {
        Arc::new(Self {
            access,
            settings,
            state: Mutex::new(State::default()),
        })
    }

    /// Dipanggil berkala. Saat dinonaktifkan, riwayat langsung dibuang.
    pub fn tick(&self) {
        if !self.settings.get().clipboard_enabled {
            *self.state() = State::default();
            return;
        }
        let Some(text) = self.access.read_text() else {
            return;
        };
        let mut st = self.state();
        if st.last.as_deref() != Some(text.as_str()) {
            st.history.push(&text);
            st.last = Some(text);
        }
    }

    pub fn list(&self, query: &str) -> Vec<ClipEntry> {
        self.state().history.search(query)
    }

    /// Salin kembali satu item ke clipboard; `false` bila item sudah tidak ada atau gagal menulis.
    pub fn copy(&self, id: u32) -> bool {
        let mut st = self.state();
        let Some(text) = st.history.get(id).map(|e| e.text.clone()) else {
            return false;
        };
        if !self.access.write_text(&text) {
            return false;
        }
        st.history.push(&text);
        st.last = Some(text);
        true
    }

    /// Taruh berkas di clipboard; bukan teks, jadi tidak masuk riwayat.
    pub fn copy_files(&self, paths: &[String]) -> bool {
        self.access.write_files(paths)
    }

    pub fn clear(&self) {
        self.state().history.clear();
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::model::settings::Settings;

    #[derive(Default)]
    struct Fake(Mutex<String>);
    impl ClipboardAccess for Fake {
        fn read_text(&self) -> Option<String> {
            Some(self.0.lock().unwrap().clone())
        }
        fn write_text(&self, text: &str) -> bool {
            *self.0.lock().unwrap() = text.to_string();
            true
        }
        fn write_files(&self, _paths: &[String]) -> bool {
            true
        }
    }

    fn fx(enabled: bool) -> (Arc<ClipboardService>, Arc<Fake>, tokio::runtime::Runtime) {
        let (settings, _island, rt) = services(Settings {
            clipboard_enabled: enabled,
            ..Default::default()
        });
        let fake = Arc::new(Fake::default());
        (ClipboardService::new(fake.clone(), settings), fake, rt)
    }

    fn copy_text(f: &Fake, t: &str) {
        *f.0.lock().unwrap() = t.into();
    }

    #[test]
    fn mencatat_perubahan_dan_menyalin_kembali() {
        let (svc, fake, _rt) = fx(true);
        copy_text(&fake, "satu");
        svc.tick();
        svc.tick(); // isi sama: tidak ganda
        copy_text(&fake, "dua");
        svc.tick();
        let list = svc.list("");
        assert_eq!(
            list.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(),
            ["dua", "satu"]
        );

        assert!(svc.copy(list[1].id));
        assert_eq!(*fake.0.lock().unwrap(), "satu");
        assert_eq!(svc.list("")[0].text, "satu");
        assert!(!svc.copy(9999));
    }

    #[test]
    fn nonaktif_tidak_mencatat_sama_sekali() {
        let (svc, fake, _rt) = fx(false);
        copy_text(&fake, "rahasia");
        svc.tick();
        assert!(svc.list("").is_empty());
    }
}
