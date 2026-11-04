use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use crate::domain::disk::{DiskEntry, DiskReport};
use crate::domain::error::DiskError;
use crate::domain::ports::DiskScanner;

/// Pemindai ukuran lewat `std::fs`. Ukuran = ukuran logis berkas (bukan ruang terpakai di disk).
/// Tautan simbolik dan junction dilewati supaya tidak berputar atau terhitung dua kali.
pub struct FsDiskScanner;

fn is_link(ft: &fs::FileType) -> bool {
    ft.is_symlink()
}

/// Jalan paralel: tiap pekerja mengambil satu folder dari tumpukan bersama, menambah ukuran berkasnya ke
/// total milik anak teratas (`.0` = indeks), dan mendorong sub-foldernya kembali. Folder yang tak bisa dibaca
/// (izin) dilewati diam-diam. Berhenti dini bila `cancel` menyala.
fn walk(
    roots: Vec<(usize, PathBuf)>,
    totals: &[AtomicU64],
    cancel: &AtomicBool,
    progress: &AtomicU64,
) {
    let pending = AtomicUsize::new(roots.len());
    let stack = Mutex::new(roots);
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().clamp(2, 8));
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while !cancel.load(Ordering::Relaxed) {
                    let item = stack.lock().unwrap_or_else(|e| e.into_inner()).pop();
                    let Some((idx, dir)) = item else {
                        if pending.load(Ordering::SeqCst) == 0 {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    };
                    let mut subdirs = Vec::new();
                    let mut bytes = 0;
                    if let Ok(rd) = fs::read_dir(&dir) {
                        for entry in rd.flatten() {
                            let Ok(ft) = entry.file_type() else { continue };
                            if is_link(&ft) {
                                continue;
                            }
                            if ft.is_dir() {
                                subdirs.push((idx, entry.path()));
                            } else if let Ok(meta) = entry.metadata() {
                                bytes += meta.len();
                                progress.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    totals[idx].fetch_add(bytes, Ordering::Relaxed);
                    // Tambah dulu baru kurangi, supaya pekerja lain tidak mengira pekerjaan sudah habis.
                    pending.fetch_add(subdirs.len(), Ordering::SeqCst);
                    stack
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .extend(subdirs);
                    pending.fetch_sub(1, Ordering::SeqCst);
                }
            });
        }
    });
}

impl DiskScanner for FsDiskScanner {
    fn drives(&self) -> Vec<String> {
        #[cfg(windows)]
        {
            (b'A'..=b'Z')
                .map(|l| format!("{}:\\", l as char))
                .filter(|d| Path::new(d).exists())
                .collect()
        }
        #[cfg(not(windows))]
        {
            vec!["/".to_string()]
        }
    }

    fn scan(
        &self,
        root: &str,
        cancel: &AtomicBool,
        progress: &AtomicU64,
    ) -> Result<DiskReport, DiskError> {
        let rd = fs::read_dir(root).map_err(|e| DiskError::Unreadable(e.to_string()))?;
        let mut entries = Vec::new();
        let mut dirs = Vec::new();
        for entry in rd.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if is_link(&ft) {
                continue;
            }
            let path = entry.path();
            let is_dir = ft.is_dir();
            if is_dir {
                dirs.push((entries.len(), path.clone()));
            } else {
                progress.fetch_add(1, Ordering::Relaxed);
            }
            entries.push(DiskEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: path.to_string_lossy().into_owned(),
                size: if is_dir {
                    0
                } else {
                    entry.metadata().map_or(0, |m| m.len())
                },
                is_dir,
            });
        }
        let totals: Vec<AtomicU64> = (0..entries.len()).map(|_| AtomicU64::new(0)).collect();
        walk(dirs, &totals, cancel, progress);
        for (e, t) in entries.iter_mut().zip(&totals) {
            if e.is_dir {
                e.size = t.load(Ordering::Relaxed);
            }
        }
        Ok(DiskReport::new(root.to_string(), entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(root: &Path) -> DiskReport {
        FsDiskScanner
            .scan(
                &root.to_string_lossy(),
                &AtomicBool::new(false),
                &AtomicU64::new(0),
            )
            .unwrap()
    }

    #[test]
    fn menjumlahkan_isi_folder_bersarang_dan_mengurutkan_terbesar() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("kecil.txt"), [0u8; 10]).unwrap();
        fs::create_dir_all(dir.path().join("besar/dalam")).unwrap();
        fs::write(dir.path().join("besar/a.bin"), [0u8; 100]).unwrap();
        fs::write(dir.path().join("besar/dalam/b.bin"), [0u8; 50]).unwrap();
        let r = scan(dir.path());
        assert_eq!(r.total, 160);
        assert_eq!(
            (r.entries[0].name.as_str(), r.entries[0].size),
            ("besar", 150)
        );
        assert!(r.entries[0].is_dir && !r.entries[1].is_dir);
    }

    #[test]
    fn folder_tak_ada_dilaporkan_dan_pembatalan_menghentikan_pemindaian() {
        assert!(matches!(
            FsDiskScanner.scan(
                "Z:\\tidak\\ada\\sama\\sekali",
                &AtomicBool::new(false),
                &AtomicU64::new(0)
            ),
            Err(DiskError::Unreadable(_))
        ));
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("x")).unwrap();
        fs::write(dir.path().join("x/f"), [0u8; 5]).unwrap();
        let cancel = AtomicBool::new(true);
        let r = FsDiskScanner
            .scan(&dir.path().to_string_lossy(), &cancel, &AtomicU64::new(0))
            .unwrap();
        assert_eq!(r.total, 0);
    }
}
