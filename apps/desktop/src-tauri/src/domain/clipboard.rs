//! Riwayat clipboard: hanya teks, terbaru di depan, tanpa duplikat, dibatasi jumlah dan ukuran.
use std::collections::VecDeque;

pub const MAX_ITEMS: usize = 20;
pub const MAX_CHARS: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipEntry {
    pub id: u32,
    pub text: String,
}

#[derive(Default)]
pub struct ClipboardHistory {
    items: VecDeque<ClipEntry>,
    next_id: u32,
}

impl ClipboardHistory {
    /// Simpan salinan baru; salinan yang sama dipindah ke depan. Kosong atau terlalu besar diabaikan.
    pub fn push(&mut self, text: &str) {
        if text.trim().is_empty() || text.chars().count() > MAX_CHARS {
            return;
        }
        self.items.retain(|e| e.text != text);
        self.next_id += 1;
        self.items.push_front(ClipEntry {
            id: self.next_id,
            text: text.to_string(),
        });
        self.items.truncate(MAX_ITEMS);
    }

    /// Cocokkan tanpa membedakan huruf besar/kecil; kata kunci kosong menampilkan semua.
    pub fn search(&self, query: &str) -> Vec<ClipEntry> {
        let q = query.trim().to_lowercase();
        self.items
            .iter()
            .filter(|e| q.is_empty() || e.text.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn get(&self, id: u32) -> Option<&ClipEntry> {
        self.items.iter().find(|e| e.id == id)
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(h: &ClipboardHistory, q: &str) -> Vec<String> {
        h.search(q).into_iter().map(|e| e.text).collect()
    }

    #[test]
    fn terbaru_di_depan_duplikat_dipindah_dan_kosong_diabaikan() {
        let mut h = ClipboardHistory::default();
        for t in ["a", "b", "  ", "", "a"] {
            h.push(t);
        }
        assert_eq!(texts(&h, ""), ["a", "b"]);
    }

    #[test]
    fn dibatasi_dua_puluh_item_dan_ukuran() {
        let mut h = ClipboardHistory::default();
        for i in 0..30 {
            h.push(&format!("item{i}"));
        }
        h.push(&"x".repeat(MAX_CHARS + 1));
        let all = texts(&h, "");
        assert_eq!(all.len(), MAX_ITEMS);
        assert_eq!(all[0], "item29");
        assert_eq!(all[19], "item10");
    }

    #[test]
    fn cari_tanpa_beda_huruf_dan_ambil_lewat_id() {
        let mut h = ClipboardHistory::default();
        h.push("Halo Dunia");
        h.push("kode 42");
        assert_eq!(texts(&h, " DUNIA "), ["Halo Dunia"]);
        let id = h.search("kode")[0].id;
        assert_eq!(h.get(id).unwrap().text, "kode 42");
        h.clear();
        assert!(h.get(id).is_none());
    }
}
