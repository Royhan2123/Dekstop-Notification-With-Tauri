use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::domain::error::StartupError;
use crate::domain::ports::{AppLauncher, InstalledApps, StartupStore};
use crate::domain::startup::{
    InstalledApp, MAX_DELAY_SECS, MAX_STARTUP_APPS, StartupApp, clean_path, is_launchable, moved,
    name_from_path,
};

/// Daftar aplikasi yang dibuka berurutan (dengan jeda) saat Windows menyala.
pub struct StartupService {
    store: Arc<dyn StartupStore>,
    launcher: Arc<dyn AppLauncher>,
    catalog: Arc<dyn InstalledApps>,
    running: AtomicBool,
}

/// Menurunkan penanda "sedang jalan" walau thread peluncur panik.
struct RunGuard(Arc<StartupService>);

impl Drop for RunGuard {
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
    }
}

impl StartupService {
    pub fn new(
        store: Arc<dyn StartupStore>,
        launcher: Arc<dyn AppLauncher>,
        catalog: Arc<dyn InstalledApps>,
    ) -> Arc<Self> {
        Arc::new(Self {
            store,
            launcher,
            catalog,
            running: AtomicBool::new(false),
        })
    }

    pub fn list(&self) -> Result<Vec<StartupApp>, StartupError> {
        self.store.list()
    }

    /// Aplikasi terpasang yang bisa dipilih.
    pub fn installed(&self) -> Vec<InstalledApp> {
        self.catalog.list()
    }

    /// Tambah satu aplikasi ke akhir daftar; `name` kosong berarti ambil dari nama berkas.
    pub fn add(&self, name: &str, path: &str) -> Result<(), StartupError> {
        let path = clean_path(path).ok_or(StartupError::EmptyPath)?;
        if !is_launchable(&path) {
            return Err(StartupError::NotApp);
        }
        if self.store.list()?.len() >= MAX_STARTUP_APPS {
            return Err(StartupError::TooMany);
        }
        let name = name.trim();
        let name = if name.is_empty() {
            name_from_path(&path)
        } else {
            name.to_string()
        };
        self.store.add(&name, &path, 0)
    }

    pub fn set_delay(&self, id: i32, delay_secs: u32) -> Result<(), StartupError> {
        self.store.set_delay(id, delay_secs.min(MAX_DELAY_SECS))
    }

    pub fn move_item(&self, id: i32, up: bool) -> Result<(), StartupError> {
        let ids: Vec<i32> = self.store.list()?.iter().map(|a| a.id).collect();
        match moved(&ids, id, up) {
            Some(order) => self.store.reorder(&order),
            None => Ok(()),
        }
    }

    pub fn remove(&self, id: i32) -> Result<(), StartupError> {
        self.store.remove(id)
    }

    /// Buka semua aplikasi di thread sendiri: tunggu jeda tiap aplikasi, lalu buka berurutan.
    /// Aplikasi yang gagal dibuka dicatat dan dilewati; panggilan kedua saat masih jalan ditolak.
    pub fn run_all(self: &Arc<Self>) -> Result<JoinHandle<()>, StartupError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(StartupError::Busy);
        }
        let guard = RunGuard(Arc::clone(self));
        let apps = self.store.list()?;
        Ok(std::thread::spawn(move || {
            let svc = Arc::clone(&guard.0);
            for app in apps {
                if app.delay_secs > 0 {
                    std::thread::sleep(Duration::from_secs(app.delay_secs.into()));
                }
                match svc.launcher.launch(&app.path) {
                    Ok(()) => tracing::info!("startup: {} dibuka", app.name),
                    Err(e) => tracing::warn!("startup: {} dilewati: {e}", app.name),
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct MemStore(Mutex<Vec<StartupApp>>);

    impl StartupStore for MemStore {
        fn list(&self) -> Result<Vec<StartupApp>, StartupError> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn add(&self, name: &str, path: &str, delay_secs: u32) -> Result<(), StartupError> {
            let mut v = self.0.lock().unwrap();
            let id = v.iter().map(|a| a.id).max().unwrap_or(0) + 1;
            v.push(StartupApp {
                id,
                name: name.into(),
                path: path.into(),
                delay_secs,
            });
            Ok(())
        }
        fn set_delay(&self, id: i32, delay_secs: u32) -> Result<(), StartupError> {
            let mut v = self.0.lock().unwrap();
            let a = v
                .iter_mut()
                .find(|a| a.id == id)
                .ok_or(StartupError::NotFound)?;
            a.delay_secs = delay_secs;
            Ok(())
        }
        fn reorder(&self, ids: &[i32]) -> Result<(), StartupError> {
            let mut v = self.0.lock().unwrap();
            v.sort_by_key(|a| ids.iter().position(|&i| i == a.id));
            Ok(())
        }
        fn remove(&self, id: i32) -> Result<(), StartupError> {
            self.0.lock().unwrap().retain(|a| a.id != id);
            Ok(())
        }
    }

    #[derive(Default)]
    struct Recorder(Mutex<Vec<String>>);

    impl AppLauncher for Recorder {
        fn launch(&self, path: &str) -> Result<(), StartupError> {
            if path.contains("rusak") {
                return Err(StartupError::Launch("tidak ada".into()));
            }
            self.0.lock().unwrap().push(path.into());
            Ok(())
        }
    }

    struct Catalog;

    impl InstalledApps for Catalog {
        fn list(&self) -> Vec<InstalledApp> {
            vec![InstalledApp {
                name: "Chrome".into(),
                path: "C:\\chrome.exe".into(),
            }]
        }
    }

    fn fixture() -> (Arc<StartupService>, Arc<Recorder>) {
        let rec = Arc::new(Recorder::default());
        (
            StartupService::new(
                Arc::new(MemStore::default()),
                rec.clone(),
                Arc::new(Catalog),
            ),
            rec,
        )
    }

    #[test]
    fn tambah_menolak_kosong_dan_batas_lalu_menamai_dari_berkas() {
        let (svc, _) = fixture();
        assert!(matches!(
            svc.add("", "  \"\" "),
            Err(StartupError::EmptyPath)
        ));
        assert!(matches!(
            svc.add("", "C:\\a.pdf"),
            Err(StartupError::NotApp)
        ));
        svc.add("", "C:\\Apps\\Code.exe").unwrap();
        svc.add(" Chrome Browser ", "C:\\chrome.exe").unwrap();
        let names: Vec<_> = svc.list().unwrap().into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["Code", "Chrome Browser"]);
        assert_eq!(svc.installed().len(), 1);
        for i in 2..MAX_STARTUP_APPS {
            svc.add("", &format!("C:\\a{i}.exe")).unwrap();
        }
        assert!(matches!(
            svc.add("", "C:\\x.exe"),
            Err(StartupError::TooMany)
        ));
    }

    #[test]
    fn jeda_dibatasi_dan_urutan_bisa_digeser() {
        let (svc, _) = fixture();
        svc.add("", "C:\\a.exe").unwrap();
        svc.add("", "C:\\b.exe").unwrap();
        svc.set_delay(1, 99_999).unwrap();
        assert_eq!(svc.list().unwrap()[0].delay_secs, MAX_DELAY_SECS);
        svc.move_item(2, true).unwrap();
        let names: Vec<_> = svc.list().unwrap().into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["b", "a"]);
    }

    #[test]
    fn menjalankan_berurutan_melewati_yang_gagal() {
        let (svc, rec) = fixture();
        for p in ["C:\\a.exe", "C:\\rusak.exe", "C:\\c.exe"] {
            svc.add("", p).unwrap();
        }
        svc.run_all().unwrap().join().unwrap();
        assert_eq!(*rec.0.lock().unwrap(), ["C:\\a.exe", "C:\\c.exe"]);
        // Selesai: boleh dijalankan lagi.
        svc.run_all().unwrap().join().unwrap();
    }

    #[test]
    fn panggilan_kedua_saat_masih_jalan_ditolak() {
        let (svc, _) = fixture();
        svc.add("", "C:\\a.exe").unwrap();
        svc.set_delay(1, 1).unwrap();
        let h = svc.run_all().unwrap();
        assert!(matches!(svc.run_all(), Err(StartupError::Busy)));
        h.join().unwrap();
    }
}
