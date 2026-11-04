/// Satu baris hasil pemindaian: berkas atau folder (ukuran folder = isi semua turunannya).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiskEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

/// Isi satu folder, terbesar di atas. `total` mencakup semua isi, bukan hanya yang ditampilkan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiskReport {
    pub path: String,
    pub total: u64,
    pub entries: Vec<DiskEntry>,
}

/// Jumlah baris terbesar yang ditampilkan.
pub const MAX_DISK_ENTRIES: usize = 40;

impl DiskReport {
    /// Susun laporan: urut dari terbesar, dipotong ke `MAX_DISK_ENTRIES`; `total` dihitung sebelum dipotong.
    pub fn new(path: String, mut entries: Vec<DiskEntry>) -> Self {
        entries.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
        let total = entries.iter().map(|e| e.size).sum();
        entries.truncate(MAX_DISK_ENTRIES);
        Self {
            path,
            total,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(name: &str, size: u64) -> DiskEntry {
        DiskEntry {
            name: name.into(),
            path: name.into(),
            size,
            is_dir: false,
        }
    }

    #[test]
    fn laporan_urut_terbesar_dan_total_mencakup_yang_terpotong() {
        let many: Vec<_> = (0..MAX_DISK_ENTRIES as u64 + 10)
            .map(|i| e(&format!("f{i:03}"), i))
            .collect();
        let sum: u64 = many.iter().map(|x| x.size).sum();
        let r = DiskReport::new("C:\\".into(), many);
        assert_eq!(r.entries.len(), MAX_DISK_ENTRIES);
        assert_eq!(r.total, sum);
        assert!(r.entries.windows(2).all(|w| w[0].size >= w[1].size));
        let tie = DiskReport::new("x".into(), vec![e("b", 5), e("a", 5)]);
        assert_eq!(tie.entries[0].name, "a");
    }
}
