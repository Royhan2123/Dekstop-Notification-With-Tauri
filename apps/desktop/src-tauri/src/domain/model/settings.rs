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

/// Yang dilakukan bila pengguna tidak merespons review dalam batas waktu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTimeoutAction {
    #[default]
    Continue,
    Cancel,
}

/// Aturan pengingat kesehatan yang aktif.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct HealthRules {
    pub continuous: bool,
    pub twenty_twenty_twenty: bool,
    pub bright_night: bool,
    pub late_night: bool,
}

impl Default for HealthRules {
    fn default() -> Self {
        Self {
            continuous: true,
            twenty_twenty_twenty: false,
            bright_night: true,
            late_night: true,
        }
    }
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
    /// Akar repo yang diawasi (hook `pre-push` terpasang).
    pub watched_repos: Vec<String>,
    pub review_timeout_action: ReviewTimeoutAction,
    pub health_enabled: bool,
    pub health_rules: HealthRules,
    /// Riwayat clipboard (di memori saja); mati secara bawaan karena clipboard bisa memuat rahasia.
    pub clipboard_enabled: bool,
    /// Id tema bawaan (palet warna pulau dan Mino).
    pub theme: String,
    /// Warna aksen buatan sendiri (`#rrggbb`); bila terisi menggantikan `theme`.
    pub custom_accent: String,
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
            watched_repos: Vec::new(),
            review_timeout_action: ReviewTimeoutAction::default(),
            health_enabled: true,
            health_rules: HealthRules::default(),
            clipboard_enabled: false,
            theme: "mint".into(),
            custom_accent: String::new(),
        }
    }
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl Settings {
    /// Rapikan nilai di luar batas; dipanggil sebelum disimpan.
    pub fn sanitized(mut self) -> Self {
        self.display_seconds = self
            .display_seconds
            .clamp(MIN_DISPLAY_SECONDS, MAX_DISPLAY_SECONDS);
        self.ai_model = self.ai_model.trim().chars().take(100).collect();
        self.theme.retain(|c| c.is_ascii_alphanumeric() || c == '-');
        self.theme.truncate(32);
        if !is_hex_color(&self.custom_accent) {
            self.custom_accent.clear();
        }
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
    fn sanitized_menolak_warna_dan_tema_tak_valid() {
        let s = Settings {
            theme: "o<c>ean!".into(),
            custom_accent: "red".into(),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.theme, "ocean");
        assert!(s.custom_accent.is_empty());
        let s = Settings {
            custom_accent: "#A1b2C3".into(),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.custom_accent, "#A1b2C3");
    }

    #[test]
    fn json_lama_tanpa_field_baru_memakai_default() {
        let s: Settings = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert_eq!(s.language, Language::En);
        assert_eq!(s.display_seconds, 8);
    }
}
