/// Satu butir di daftar pulau: tugas (bisa dicentang) atau catatan (judul + isi).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub done: bool,
    /// Catatan, bukan tugas: tidak dicentang, tetapi punya isi.
    pub note: bool,
    pub body: String,
}

pub const MAX_TASKS: usize = 200;
pub const MAX_TITLE_CHARS: usize = 120;
pub const MAX_BODY_CHARS: usize = 4000;

/// Rapikan judul dari kotak input: spasi berlebih dibuang, dipotong ke batas; kosong ditolak.
pub fn clean_title(raw: &str) -> Option<String> {
    let t = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let t: String = t.chars().take(MAX_TITLE_CHARS).collect();
    (!t.is_empty()).then_some(t)
}

/// Isi catatan: baris baru dipertahankan, spasi di ujung dibuang, dipotong ke batas.
pub fn clean_body(raw: &str) -> String {
    raw.trim_end().chars().take(MAX_BODY_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn judul_dirapikan_dipotong_dan_kosong_ditolak() {
        assert_eq!(clean_title("  beli   susu \n"), Some("beli susu".into()));
        assert_eq!(clean_title("   "), None);
        assert_eq!(
            clean_title(&"a".repeat(500)).unwrap().chars().count(),
            MAX_TITLE_CHARS
        );
    }

    #[test]
    fn isi_catatan_menjaga_baris_baru_dan_dipotong() {
        assert_eq!(clean_body("a\n\nb  \n"), "a\n\nb");
        assert_eq!(clean_body(""), "");
        assert_eq!(
            clean_body(&"x".repeat(9000)).chars().count(),
            MAX_BODY_CHARS
        );
    }
}
