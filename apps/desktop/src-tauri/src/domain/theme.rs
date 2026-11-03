//! Aturan tema yang diambil dari luar: URL berkas tema dan format berkasnya.

/// Batas ukuran berkas tema dari URL (sama dengan impor dari berkas di Setelan).
pub const MAX_THEME_BYTES: usize = 600_000;
const MAX_URL_LEN: usize = 2048;

/// Hanya `https` ke nama host biasa. Alamat IP, `localhost`, dan nama internal ditolak supaya
/// tautan dari orang lain tidak bisa membuat aplikasi membuka layanan di jaringan pengguna.
pub fn is_allowed_theme_url(url: &str) -> bool {
    if url.len() > MAX_URL_LEN || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return false;
    }
    let (host, port) = authority.split_once(':').unwrap_or((authority, ""));
    if !port.chars().all(|c| c.is_ascii_digit()) || host.is_empty() {
        return false;
    }
    let host = host.to_ascii_lowercase();
    let ip_like = host.chars().all(|c| c.is_ascii_digit() || c == '.') || host.starts_with('[');
    let internal = host == "localhost"
        || !host.contains('.')
        || ["localhost", "local", "internal", "lan", "home", "corp"]
            .iter()
            .any(|t| host.ends_with(&format!(".{t}")));
    !ip_like && !internal
}

/// Apakah teks adalah berkas tema Isla (JSON dengan penanda `isla_theme` versi 1).
pub fn is_theme_file(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .is_some_and(|v| v["isla_theme"] == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_https_ke_nama_host_biasa_diterima() {
        for ok in [
            "https://example.com/tema.json",
            "https://raw.githubusercontent.com/u/r/main/a.isla-theme.json?x=1",
            "https://gist.github.com:443/a",
        ] {
            assert!(is_allowed_theme_url(ok), "{ok}");
        }
    }

    #[test]
    fn url_berbahaya_atau_internal_ditolak() {
        for bad in [
            "",
            "http://example.com/a.json",
            "ftp://example.com/a",
            "file:///C:/a.json",
            "https://localhost/a",
            "https://foo.localhost/a",
            "https://127.0.0.1/a",
            "https://192.168.1.5/a",
            "https://[::1]/a",
            "https://intranet/a",
            "https://nas.local/a",
            "https://user:pw@example.com/a",
            "https://example.com:abc/a",
            "https://exa mple.com/a",
            "https:///a",
        ] {
            assert!(!is_allowed_theme_url(bad), "{bad}");
        }
        let long = format!("https://example.com/{}", "a".repeat(3000));
        assert!(!is_allowed_theme_url(&long));
    }

    #[test]
    fn berkas_tema_dikenali_dari_penanda_versi() {
        assert!(is_theme_file(r##"{"isla_theme":1,"theme":"mint"}"##));
        assert!(!is_theme_file(r#"{"isla_theme":2}"#));
        assert!(!is_theme_file(r#"{"theme":"mint"}"#));
        assert!(!is_theme_file("<html>"));
    }
}
