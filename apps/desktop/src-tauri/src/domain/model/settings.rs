use serde::{Deserialize, Serialize};
use specta::Type;

// Catatan: derive serde/specta di sini hanya refleksi tipe (tanpa I/O); dipakai agar
// penyimpanan JSON dan binding TypeScript tidak perlu DTO salinan.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Id,
    En,
}

/// "Otak" AI untuk review kode (dipakai mulai Fase 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    #[default]
    ClaudeApi,
    Ollama,
    ClaudeCli,
}

pub const MIN_DISPLAY_SECONDS: u32 = 2;
pub const MAX_DISPLAY_SECONDS: u32 = 60;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub reduce_motion: bool,
    /// Lama notifikasi (kesehatan/info) tampil sebelum menutup sendiri.
    pub display_seconds: u32,
    pub autostart: bool,
    pub ai_provider: AiProvider,
    pub ai_model: String,
    pub onboarded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            reduce_motion: false,
            display_seconds: 8,
            autostart: false,
            ai_provider: AiProvider::default(),
            ai_model: String::new(),
            onboarded: false,
        }
    }
}

impl Settings {
    /// Rapikan nilai di luar batas; dipanggil sebelum disimpan.
    pub fn sanitized(mut self) -> Self {
        self.display_seconds = self
            .display_seconds
            .clamp(MIN_DISPLAY_SECONDS, MAX_DISPLAY_SECONDS);
        self.ai_model = self.ai_model.trim().chars().take(100).collect();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_membatasi_durasi_dan_merapikan_model() {
        let s = Settings {
            display_seconds: 0,
            ai_model: format!("  {}  ", "m".repeat(300)),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.display_seconds, MIN_DISPLAY_SECONDS);
        assert_eq!(s.ai_model.len(), 100);
        let s = Settings {
            display_seconds: 9999,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.display_seconds, MAX_DISPLAY_SECONDS);
    }

    #[test]
    fn json_lama_tanpa_field_baru_memakai_default() {
        let s: Settings = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert_eq!(s.language, Language::En);
        assert_eq!(s.display_seconds, 8);
    }
}
