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

/// Jarvis punya persona dan alatnya sendiri. Petunjuk sistem Isla dikirim terpisah (`system`) agar
/// tidak ikut dihitung saat Jarvis memilih model: kalau digabung, pertanyaan pendek dianggap rumit.
pub struct JarvisClient;

impl AiClient for JarvisClient {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let ep = endpoint()?;
        let req = http()?
            .post(format!("http://127.0.0.1:{}/v1/ask", ep.port))
            .bearer_auth(&ep.token)
            .json(&json!({ "text": user, "system": system }));
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

/// Setelan AI Jarvis (`/v1/ai`): penyedia otak Jarvis sendiri (Ollama lokal atau API online).
/// Kunci API hanya dikirim ke Jarvis (loopback) dan tidak pernah dibaca kembali.
pub fn ai_get() -> Result<Value, AiError> {
    let ep = endpoint()?;
    let req = http()?
        .get(format!("http://127.0.0.1:{}/v1/ai", ep.port))
        .bearer_auth(&ep.token);
    call(req, "Jarvis")
}

pub fn ai_set(body: &Value) -> Result<Value, AiError> {
    let ep = endpoint()?;
    let req = http()?
        .put(format!("http://127.0.0.1:{}/v1/ai", ep.port))
        .bearer_auth(&ep.token)
        .json(body);
    call(req, "Jarvis")
}

/// Satu pertanyaan uji ke penyedia aktif Jarvis; `Err` berisi alasan yang aman ditampilkan.
pub fn ai_test() -> Result<String, AiError> {
    let ep = endpoint()?;
    let req = http()?
        .post(format!("http://127.0.0.1:{}/v1/ai/test", ep.port))
        .bearer_auth(&ep.token);
    let v = call(req, "Jarvis")?;
    if v.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(v.get("reply")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string())
    } else {
        let why = v.get("error").and_then(Value::as_str).unwrap_or("gagal");
        Err(AiError::Unavailable(why.to_string()))
    }
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

    /// Pertanyaan yang sama persis dengan jalur "Tanya AI" Isla (penyedia Jarvis → `/v1/ask`).
    /// Hanya pertanyaan baca-saja: tidak mengubah memori atau komputer pengguna.
    #[test]
    #[ignore = "butuh Jarvis dan Ollama berjalan"]
    fn jarvis_asli_menjawab_beberapa_pertanyaan_lewat_jalur_isla() {
        let cases: [(&str, &[&str]); 6] = [
            ("Apa ibu kota Jepang?", &["tokyo", "tokio"]),
            ("Berapa 12 kali 12?", &["144", "seratus empat puluh empat"]),
            ("Siapa namamu?", &["jarvis"]),
            (
                "Terjemahkan ke bahasa Inggris: selamat pagi",
                &["good morning"],
            ),
            ("What is the capital of France?", &["paris"]),
            ("Jam berapa sekarang?", &["jam", "pukul"]),
        ];
        let mut gagal = Vec::new();
        for (tanya, kata) in cases {
            let t = std::time::Instant::now();
            let reply = JarvisClient
                .complete("", tanya)
                .unwrap_or_else(|e| e.to_string());
            println!("[{:>5.1}s] {tanya} -> {reply}", t.elapsed().as_secs_f32());
            if !kata.iter().any(|k| reply.to_lowercase().contains(k)) {
                gagal.push(format!("{tanya} -> {reply}"));
            }
        }
        assert!(gagal.is_empty(), "jawaban tak sesuai: {gagal:#?}");
    }

    /// Setelan AI Jarvis lewat API (yang dipakai bagian "Otak Jarvis" di Setelan Isla).
    #[test]
    #[ignore = "butuh Jarvis berjalan"]
    fn jarvis_asli_setelan_ai_terbaca_tanpa_kunci_dan_bisa_diuji() {
        let v = ai_get().unwrap();
        assert_eq!(v["provider"], "ollama");
        assert!(v["providers"].as_array().unwrap().len() >= 5);
        assert!(!v.to_string().contains("api_key"));
        assert!(!ai_test().unwrap().is_empty());
        // penyedia online tanpa kunci ditolak (422) dan tidak mengubah apa pun
        let err = ai_set(&json!({"provider": "openai", "fast_model": "m"})).unwrap_err();
        assert!(err.to_string().contains("422"), "{err}");
        assert_eq!(ai_get().unwrap()["provider"], "ollama");
    }
}
