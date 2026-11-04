use std::io::Read;
use std::time::Duration;

use crate::domain::error::ThemeError;
use crate::domain::ports::ThemeFetcher;
use crate::domain::theme::{MAX_THEME_BYTES, is_public_https_url};

const MAX_REDIRECTS: usize = 3;

/// Unduh teks dari URL https publik. Pengalihan hanya diikuti ke URL yang lolos aturan yang sama,
/// ukuran dibatasi `max_bytes`, dan waktu tunggu pendek.
pub fn https_get(url: &str, max_bytes: usize) -> Result<String, ThemeError> {
    let unavailable = |e: &dyn std::fmt::Display| ThemeError::Unavailable(e.to_string());
    let policy = reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() > MAX_REDIRECTS || !is_public_https_url(attempt.url().as_str())
        {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(policy)
        .user_agent(concat!("IslaDesk/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| unavailable(&e))?;
    let response = client.get(url).send().map_err(|e| unavailable(&e))?;
    if !response.status().is_success() {
        return Err(ThemeError::Unavailable(format!(
            "server menjawab {}",
            response.status()
        )));
    }
    let mut body = Vec::new();
    response
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut body)
        .map_err(|e| unavailable(&e))?;
    if body.len() > max_bytes {
        return Err(ThemeError::Invalid("berkas terlalu besar".into()));
    }
    String::from_utf8(body).map_err(|_| ThemeError::Invalid("bukan teks".into()))
}

/// Mengunduh berkas tema lewat HTTPS.
pub struct HttpThemeFetcher;

impl ThemeFetcher for HttpThemeFetcher {
    fn fetch(&self, url: &str) -> Result<String, ThemeError> {
        https_get(url, MAX_THEME_BYTES)
    }
}
