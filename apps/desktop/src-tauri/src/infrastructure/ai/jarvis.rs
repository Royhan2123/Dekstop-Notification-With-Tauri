//! Penyedia "Jarvis": pertanyaan dijawab asisten Jarvis di komputer ini (API lokal 127.0.0.1).
//! Alamat dan token dibaca dari `api.json` yang ditulis Jarvis (butuh `ui.api.enabled: true`).
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{Value, json};

use super::{call, http};
use crate::domain::error::AiError;
use crate::domain::ports::AiClient;

#[derive(Deserialize)]
struct Endpoint {
    port: u16,
    token: String,
}

/// Lokasi `api.json` Jarvis: folder data lokal pengguna (sama dengan `platformdirs` di Jarvis).
pub fn api_file() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support");
    #[cfg(windows)]
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    #[cfg(all(not(target_os = "macos"), not(windows)))]
    let base = match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".local/share"),
    };
    Some(base.join("jarvis").join("api.json"))
}

fn endpoint() -> Result<Endpoint, AiError> {
    let unavailable = || {
        AiError::Unavailable(
            "Jarvis tidak berjalan atau API lokalnya mati (ui.api.enabled: true di config.yaml)"
                .into(),
        )
    };
    let path = api_file().ok_or_else(unavailable)?;
    let bytes = std::fs::read(path).map_err(|_| unavailable())?;
    serde_json::from_slice(&bytes).map_err(|_| unavailable())
}

/// Jarvis punya persona dan alatnya sendiri, jadi petunjuk sistem Isla digabung ke pertanyaan.
pub struct JarvisClient;

impl AiClient for JarvisClient {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let ep = endpoint()?;
        let text = if system.trim().is_empty() {
            user.to_string()
        } else {
            format!("{system}\n\n{user}")
        };
        let req = http()?
            .post(format!("http://127.0.0.1:{}/v1/ask", ep.port))
            .bearer_auth(&ep.token)
            .json(&json!({ "text": text }));
        let v: Value = call(req, "Jarvis")?;
        v.get("reply")
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(AiError::BadResponse)
    }
}

/// Jawab kartu izin agen dari Jarvis (`POST /v1/approvals/{id}`); `true` bila diterima.
pub fn decide_approval(id: &str, allow: bool) -> Result<(), AiError> {
    let ep = endpoint()?;
    let req = http()?
        .post(format!("http://127.0.0.1:{}/v1/approvals/{id}", ep.port))
        .bearer_auth(&ep.token)
        .json(&json!({ "allow": allow }));
    call(req, "Jarvis").map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lokasi_api_json_ada_di_folder_jarvis() {
        let p = api_file().expect("folder data tersedia");
        assert!(p.ends_with("jarvis/api.json") || p.ends_with("jarvis\\api.json"));
    }

    #[test]
    fn tanpa_berkas_galatnya_menunjuk_cara_menyalakan_api() {
        // api.json bisa saja ada di mesin pengembang; yang diuji hanya bentuk pesan bila tidak ada.
        if api_file().is_some_and(|p| p.exists()) {
            return;
        }
        let err = JarvisClient.complete("", "halo").unwrap_err().to_string();
        assert!(err.contains("ui.api.enabled"), "{err}");
    }

    /// Butuh Jarvis sungguhan berjalan dengan `ui.api.enabled: true`:
    /// `cargo test jarvis_asli -- --ignored`.
    #[test]
    #[ignore = "butuh Jarvis dan Ollama berjalan"]
    fn jarvis_asli_menjawab_dan_menolak_izin_tak_dikenal() {
        let reply = JarvisClient.complete("", "Apa ibu kota Jepang?").unwrap();
        assert!(
            reply.to_lowercase().contains("tokyo") || reply.contains("Tokio"),
            "{reply}"
        );
        let err = decide_approval("tidak-ada", true).unwrap_err().to_string();
        assert!(err.contains("404"), "{err}");
    }
}
