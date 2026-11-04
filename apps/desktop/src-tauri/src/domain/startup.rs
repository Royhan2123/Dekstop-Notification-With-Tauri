/// Aplikasi yang dibuka otomatis saat Windows menyala; urutan daftar = urutan dibuka.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupApp {
    pub id: i32,
    pub name: String,
    pub path: String,
    /// Jeda (detik) sebelum aplikasi ini dibuka, dihitung dari aplikasi sebelumnya.
    pub delay_secs: u32,
}

/// Aplikasi terpasang yang bisa dipilih (dari daftar program Windows).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
}

pub const MAX_STARTUP_APPS: usize = 20;
pub const MAX_DELAY_SECS: u32 = 300;

/// Path dari frontend: spasi dan tanda kutip di ujung dibuang; kosong ditolak.
pub fn clean_path(raw: &str) -> Option<String> {
    let p = raw.trim().trim_matches('"').trim();
    (!p.is_empty()).then(|| p.to_string())
}

/// Hanya program (.exe) atau pintasannya (.lnk) yang boleh masuk daftar startup.
pub fn is_launchable(path: &str) -> bool {
    let p = path.to_ascii_lowercase();
    p.ends_with(".exe") || p.ends_with(".lnk")
}

/// Nama tampilan = nama berkas tanpa ekstensi ("C:\\...\\chrome.exe" -> "chrome").
pub fn name_from_path(path: &str) -> String {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.rsplit_once('.')
        .map_or(file, |(stem, _)| stem)
        .to_string()
}

/// Urutan baru setelah satu butir naik atau turun satu langkah; `None` bila sudah di ujung atau tak ada.
pub fn moved(ids: &[i32], id: i32, up: bool) -> Option<Vec<i32>> {
    let i = ids.iter().position(|&x| x == id)?;
    let j = if up { i.checked_sub(1)? } else { i + 1 };
    (j < ids.len()).then(|| {
        let mut out = ids.to_vec();
        out.swap(i, j);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_dirapikan_dan_nama_diambil_dari_berkas() {
        assert_eq!(
            clean_path("  \"C:\\Apps\\a b.exe\" "),
            Some("C:\\Apps\\a b.exe".into())
        );
        assert_eq!(clean_path("  \"\" "), None);
        assert_eq!(
            name_from_path("C:\\Program Files\\Google\\chrome.exe"),
            "chrome"
        );
        assert_eq!(name_from_path("D:/x/code.cmd"), "code");
        assert_eq!(name_from_path("tanpa-ekstensi"), "tanpa-ekstensi");
    }

    #[test]
    fn hanya_exe_dan_pintasan_yang_bisa_dijalankan() {
        assert!(is_launchable("C:\\A\\Chrome.EXE"));
        assert!(is_launchable("C:\\Menu\\App.lnk"));
        assert!(!is_launchable("C:\\a\\skrip.bat"));
        assert!(!is_launchable("C:\\a\\dokumen.pdf"));
    }

    #[test]
    fn urutan_bertukar_dan_berhenti_di_ujung() {
        let ids = [1, 2, 3];
        assert_eq!(moved(&ids, 2, true), Some(vec![2, 1, 3]));
        assert_eq!(moved(&ids, 2, false), Some(vec![1, 3, 2]));
        assert_eq!(moved(&ids, 1, true), None);
        assert_eq!(moved(&ids, 3, false), None);
        assert_eq!(moved(&ids, 9, true), None);
    }
}
