use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use crate::domain::disk::DiskReport;
use crate::domain::ports::DiskScanner;

/// Keadaan pemindaian terakhir untuk ditampilkan (di-poll oleh UI selama `running`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiskStatus {
    pub running: bool,
    pub path: String,
    pub files_scanned: u64,
    pub report: Option<DiskReport>,
    pub error: Option<String>,
}

struct Job {
    id: u64,
    path: String,
    cancel: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
    running: bool,
    result: Option<Result<DiskReport, String>>,
}

/// Pemindai ukuran folder satu per satu; pemindaian baru membatalkan yang lama. Hanya membaca, tidak menghapus.
pub struct DiskService {
    scanner: Arc<dyn DiskScanner>,
    job: Mutex<Option<Job>>,
}

impl DiskService {
    pub fn new(scanner: Arc<dyn DiskScanner>) -> Arc<Self> {
        Arc::new(Self {
            scanner,
            job: Mutex::new(None),
        })
    }

    fn job(&self) -> MutexGuard<'_, Option<Job>> {
        self.job.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn drives(&self) -> Vec<String> {
        self.scanner.drives()
    }

    /// Mulai memindai `path` di thread sendiri dan batalkan pemindaian sebelumnya.
    pub fn start(self: &Arc<Self>, path: &str) -> JoinHandle<()> {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU64::new(0));
        let id = {
            let mut slot = self.job();
            let id = slot.as_ref().map_or(1, |j| {
                j.cancel.store(true, Ordering::Relaxed);
                j.id + 1
            });
            *slot = Some(Job {
                id,
                path: path.to_string(),
                cancel: cancel.clone(),
                progress: progress.clone(),
                running: true,
                result: None,
            });
            id
        };
        let svc = Arc::clone(self);
        let path = path.to_string();
        std::thread::spawn(move || {
            let result = svc
                .scanner
                .scan(&path, &cancel, &progress)
                .map_err(|e| e.to_string());
            // Hasil pemindaian yang sudah digantikan dibuang.
            if let Some(job) = svc.job().as_mut().filter(|j| j.id == id) {
                job.running = false;
                job.result = Some(result);
            }
        })
    }

    pub fn status(&self) -> DiskStatus {
        match self.job().as_ref() {
            None => DiskStatus::default(),
            Some(j) => {
                let (report, error) = match &j.result {
                    Some(Ok(r)) => (Some(r.clone()), None),
                    Some(Err(e)) => (None, Some(e.clone())),
                    None => (None, None),
                };
                DiskStatus {
                    running: j.running,
                    path: j.path.clone(),
                    files_scanned: j.progress.load(Ordering::Relaxed),
                    report,
                    error,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::domain::disk::DiskEntry;
    use crate::domain::error::DiskError;

    struct Fake;

    impl DiskScanner for Fake {
        fn drives(&self) -> Vec<String> {
            vec!["C:\\".into()]
        }
        fn scan(
            &self,
            root: &str,
            cancel: &AtomicBool,
            progress: &AtomicU64,
        ) -> Result<DiskReport, DiskError> {
            if root == "buruk" {
                return Err(DiskError::Unreadable("ditolak".into()));
            }
            if root == "lambat" {
                for _ in 0..200 {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            progress.store(7, Ordering::Relaxed);
            Ok(DiskReport::new(
                root.into(),
                vec![DiskEntry {
                    name: "a".into(),
                    path: format!("{root}\\a"),
                    size: 3,
                    is_dir: false,
                }],
            ))
        }
    }

    #[test]
    fn hasil_dan_galat_tampil_di_status() {
        let svc = DiskService::new(Arc::new(Fake));
        assert_eq!(svc.status(), DiskStatus::default());
        svc.start("C:\\").join().unwrap();
        let s = svc.status();
        assert!(!s.running && s.error.is_none());
        assert_eq!(s.files_scanned, 7);
        assert_eq!(s.report.unwrap().total, 3);
        svc.start("buruk").join().unwrap();
        let s = svc.status();
        assert!(s.report.is_none() && s.error.unwrap().contains("ditolak"));
    }

    #[test]
    fn pemindaian_baru_membatalkan_dan_menggantikan_yang_lama() {
        let svc = DiskService::new(Arc::new(Fake));
        let slow = svc.start("lambat");
        assert!(svc.status().running);
        svc.start("D:\\").join().unwrap();
        slow.join().unwrap();
        // Hasil "lambat" yang terlambat tidak menimpa "D:\\".
        assert_eq!(svc.status().path, "D:\\");
        assert_eq!(svc.status().report.unwrap().path, "D:\\");
    }
}
