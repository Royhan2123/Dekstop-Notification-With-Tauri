/// Satu tugas di daftar sederhana pulau.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub done: bool,
}

pub const MAX_TASKS: usize = 200;
pub const MAX_TITLE_CHARS: usize = 120;

/// Rapikan judul dari kotak input: spasi berlebih dibuang, dipotong ke batas; kosong ditolak.
pub fn clean_title(raw: &str) -> Option<String> {
    let t = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let t: String = t.chars().take(MAX_TITLE_CHARS).collect();
    (!t.is_empty()).then_some(t)
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
}
