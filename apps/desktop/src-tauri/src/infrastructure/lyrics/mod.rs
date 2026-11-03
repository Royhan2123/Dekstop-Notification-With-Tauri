use std::time::Duration;

use crate::domain::error::LyricsError;
use crate::domain::lyrics::Candidate;
use crate::domain::ports::LyricsSource;

const API: &str = "https://lrclib.net/api/search";
const ATTEMPTS: u32 = 3;

/// Pencarian lirik di LRCLIB (layanan terbuka tanpa API key). Servernya kadang menjawab "sibuk",
/// jadi ada percobaan ulang bertahap; kegagalan akhir dilaporkan sebagai `Unavailable`.
pub struct Lrclib;

/// Baca larik hasil pencarian. Jawaban non-larik (mis. `{"name":"ServerOverload"}`) dianggap galat.
fn parse_candidates(json: &str) -> Result<Vec<Candidate>, LyricsError> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|_| LyricsError::Unavailable("jawaban tidak bisa dibaca".into()))?;
    let Some(items) = value.as_array() else {
        let why = value["message"].as_str().unwrap_or("jawaban tidak terduga");
        return Err(LyricsError::Unavailable(why.into()));
    };
    Ok(items
        .iter()
        .map(|i| Candidate {
            artist: i["artistName"].as_str().unwrap_or_default().to_string(),
            track: i["trackName"].as_str().unwrap_or_default().to_string(),
            duration_secs: i["duration"].as_f64().unwrap_or(0.0),
            synced: i["syncedLyrics"].as_str().map(str::to_string),
        })
        .collect())
}

fn once(client: &reqwest::blocking::Client, query: &str) -> Result<Vec<Candidate>, LyricsError> {
    let url = reqwest::Url::parse_with_params(API, [("q", query)])
        .map_err(|e| LyricsError::Unavailable(e.to_string()))?;
    let body = client
        .get(url)
        .send()
        .and_then(|r| r.text())
        .map_err(|e| LyricsError::Unavailable(e.to_string()))?;
    parse_candidates(&body)
}

impl LyricsSource for Lrclib {
    fn search(&self, query: &str) -> Result<Vec<Candidate>, LyricsError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent(concat!("IslaDesk/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| LyricsError::Unavailable(e.to_string()))?;
        let mut last = LyricsError::Unavailable("tidak ada percobaan".into());
        for attempt in 1..=ATTEMPTS {
            match once(&client, query) {
                Ok(found) => return Ok(found),
                Err(e) => last = e,
            }
            if attempt < ATTEMPTS {
                std::thread::sleep(Duration::from_millis(800 * u64::from(attempt)));
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn larik_hasil_dibaca_dan_lirik_polos_tidak_dianggap_berwaktu() {
        let json = r#"[
            {"id":1,"duration":200.4,"syncedLyrics":"[00:01.00]a","plainLyrics":"a"},
            {"id":2,"duration":199,"syncedLyrics":null,"plainLyrics":"b"},
            {"id":3}
        ]"#;
        let c = parse_candidates(json).unwrap();
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].synced.as_deref(), Some("[00:01.00]a"));
        assert_eq!(c[1].synced, None);
        assert_eq!(c[2].duration_secs, 0.0);
    }

    #[test]
    fn server_sibuk_dan_sampah_menjadi_galat() {
        let busy = r#"{"message":"The server is busy","name":"ServerOverload"}"#;
        let err = parse_candidates(busy).unwrap_err().to_string();
        assert!(err.contains("busy"), "{err}");
        assert!(parse_candidates("<html>").is_err());
    }
}
