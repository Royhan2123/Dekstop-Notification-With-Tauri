//! Aturan jembatan ke ekstensi browser "Isla Link": perintah yang sah dan asal yang dipercaya.

pub const MIN_RATE: f64 = 0.25;
pub const MAX_RATE: f64 = 2.0;
pub const MAX_QUERY_CHARS: usize = 200;
pub const MAX_BROWSER_SESSIONS: usize = 8;
pub const MAX_ARTWORK_URL_CHARS: usize = 500;
const MAX_DURATION_MS: f64 = 24.0 * 3_600_000.0;
pub const BROWSER_PREFIX: &str = "browser:";

use crate::domain::model::MediaCommand;

/// Perintah dari pulau ke ekstensi.
#[derive(Clone, Debug, PartialEq)]
pub enum BridgeCommand {
    SetRate(f64),
    Search(String),
    /// Kontrol pemutar di tab browser tertentu.
    MediaControl {
        tab_id: u64,
        action: MediaCommand,
    },
}

/// Satu tab browser yang memutar media, sudah divalidasi.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserMedia {
    pub tab_id: u64,
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub position_ms: u32,
    pub duration_ms: u32,
    pub rate: f64,
    /// Hanya URL https; sampul diunduh oleh aplikasi, bukan ekstensi.
    pub artwork: Option<String>,
}

/// ID sesi untuk tab browser, dipakai di `MediaInfo::session_id`.
pub fn session_id(tab_id: u64) -> String {
    format!("{BROWSER_PREFIX}{tab_id}")
}

/// Kebalikan `session_id`; `None` bila bukan sesi browser.
pub fn tab_of_session(id: &str) -> Option<u64> {
    id.strip_prefix(BROWSER_PREFIX)?.parse().ok()
}

fn tidy_text(s: &str) -> String {
    let flat: String = s.chars().filter(|c| !c.is_control()).collect();
    flat.trim().chars().take(MAX_QUERY_CHARS).collect()
}

fn clamp_ms(v: f64) -> u32 {
    if v.is_finite() {
        v.clamp(0.0, MAX_DURATION_MS) as u32
    } else {
        0
    }
}

/// Data dari ekstensi tidak dipercaya: teks dirapikan, angka dibatasi, sampul harus https. Tanpa
/// judul = bukan sesi yang layak tampil.
#[allow(clippy::too_many_arguments)]
pub fn sanitize_media(
    tab_id: u64,
    title: &str,
    artist: &str,
    playing: bool,
    position_ms: f64,
    duration_ms: f64,
    rate: f64,
    artwork: &str,
) -> Option<BrowserMedia> {
    let title = tidy_text(title);
    if title.is_empty() {
        return None;
    }
    let art = artwork.trim();
    let artwork = (art.starts_with("https://")
        && art.len() <= MAX_ARTWORK_URL_CHARS
        && !art.chars().any(|c| c.is_control() || c.is_whitespace()))
    .then(|| art.to_string());
    Some(BrowserMedia {
        tab_id,
        title,
        artist: tidy_text(artist),
        playing,
        position_ms: clamp_ms(position_ms),
        duration_ms: clamp_ms(duration_ms),
        rate: clamp_rate(rate).unwrap_or(1.0),
        artwork,
    })
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
    fn data_media_dari_ekstensi_dirapikan_dan_dibatasi() {
        let m = sanitize_media(
            7,
            " Judul\n\u{0}Baru ",
            "Artis",
            true,
            f64::NAN,
            1e12,
            9.0,
            "https://i.ytimg.com/vi/x/hq.jpg",
        )
        .unwrap();
        assert_eq!(m.title, "JudulBaru");
        assert_eq!((m.position_ms, m.rate), (0, 2.0));
        assert_eq!(m.duration_ms, 86_400_000);
        assert!(m.artwork.is_some());
        assert!(sanitize_media(1, "  ", "a", true, 0.0, 0.0, 1.0, "").is_none());
        for bad in ["http://x/a.jpg", "javascript:alert(1)", "https://x/a b", ""] {
            let m = sanitize_media(1, "t", "a", true, 0.0, 0.0, 1.0, bad).unwrap();
            assert!(m.artwork.is_none(), "{bad}");
        }
    }

    #[test]
    fn id_sesi_browser_bolak_balik() {
        assert_eq!(tab_of_session(&session_id(42)), Some(42));
        assert_eq!(tab_of_session("Music"), None);
        assert_eq!(tab_of_session("browser:abc"), None);
    }

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
