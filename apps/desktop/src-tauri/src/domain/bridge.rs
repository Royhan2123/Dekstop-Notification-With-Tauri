//! Aturan jembatan ke ekstensi browser "Isla Link": perintah yang sah dan asal yang dipercaya.

pub const MIN_RATE: f64 = 0.25;
pub const MAX_RATE: f64 = 2.0;
pub const MAX_QUERY_CHARS: usize = 200;

/// Perintah dari pulau ke ekstensi.
#[derive(Clone, Debug, PartialEq)]
pub enum BridgeCommand {
    SetRate(f64),
    Search(String),
}

/// Kecepatan dibatasi 0,25x-2x dan dibulatkan ke 0,05; bukan angka (NaN/tak hingga) ditolak.
pub fn clamp_rate(rate: f64) -> Option<f64> {
    rate.is_finite()
        .then(|| ((rate.clamp(MIN_RATE, MAX_RATE)) * 20.0).round() / 20.0)
}

/// Kata kunci pencarian yang rapi; kosong atau berisi karakter kontrol ditolak.
pub fn clean_query(query: &str) -> Option<String> {
    let q: String = query.trim().chars().take(MAX_QUERY_CHARS).collect();
    (!q.is_empty() && !q.chars().any(char::is_control)).then_some(q)
}

/// Hanya ekstensi browser yang boleh tersambung; halaman web biasa (http/https) tidak.
pub fn is_extension_origin(origin: &str) -> bool {
    ["chrome-extension://", "moz-extension://"].iter().any(|p| {
        origin
            .strip_prefix(p)
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kecepatan_dibatasi_dibulatkan_dan_nan_ditolak() {
        assert_eq!(clamp_rate(1.0), Some(1.0));
        assert_eq!(clamp_rate(0.1), Some(0.25));
        assert_eq!(clamp_rate(9.0), Some(2.0));
        assert_eq!(clamp_rate(1.23), Some(1.25));
        assert_eq!(clamp_rate(f64::NAN), None);
        assert_eq!(clamp_rate(f64::INFINITY), None);
    }

    #[test]
    fn kata_kunci_dirapikan_dan_dipotong() {
        assert_eq!(clean_query("  lofi  "), Some("lofi".into()));
        assert_eq!(clean_query("   "), None);
        assert_eq!(clean_query("a\nb"), None);
        assert_eq!(
            clean_query(&"x".repeat(500)).unwrap().chars().count(),
            MAX_QUERY_CHARS
        );
    }

    #[test]
    fn hanya_asal_ekstensi_yang_dipercaya() {
        assert!(is_extension_origin("chrome-extension://abcdefgh"));
        assert!(is_extension_origin("moz-extension://1234-5678"));
        for bad in [
            "https://evil.example",
            "http://localhost:1420",
            "chrome-extension://",
            "chrome-extension://a/b",
            "",
        ] {
            assert!(!is_extension_origin(bad), "{bad}");
        }
    }
}
