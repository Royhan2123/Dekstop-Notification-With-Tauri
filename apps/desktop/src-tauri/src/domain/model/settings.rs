use crate::domain::theme::is_public_https_url;
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
    Gemini,
    DeepSeek,
    Grok,
    OpenAi,
    OpenRouter,
    Groq,
    Ollama,
    ClaudeCli,
}

impl AiProvider {
    /// Nama penyedia untuk ditampilkan ke pengguna.
    pub fn label(self) -> &'static str {
        match self {
            Self::ClaudeApi => "Claude",
            Self::Gemini => "Gemini",
            Self::DeepSeek => "DeepSeek",
            Self::Grok => "Grok",
            Self::OpenAi => "OpenAI",
            Self::OpenRouter => "OpenRouter",
            Self::Groq => "Groq",
            Self::Ollama => "Ollama",
            Self::ClaudeCli => "Claude CLI",
        }
    }

    /// Model yang dipakai bila pengguna tidak memilih; `None` bila penyedia menentukannya sendiri.
    pub fn default_model(self) -> Option<&'static str> {
        match self {
            Self::ClaudeApi => Some("claude-sonnet-5-5"),
            Self::Ollama | Self::ClaudeCli => None,
            other => other.openai_compatible().map(|(_, model)| model),
        }
    }

    /// "Claude · claude-sonnet-5-5": penyedia dan model yang benar-benar dipakai untuk `model` pilihan pengguna.
    pub fn describe(self, model: &str) -> String {
        let model = model.trim();
        match (model.is_empty(), self.default_model()) {
            (false, _) => format!("{} · {model}", self.label()),
            (true, Some(default)) => format!("{} · {default}", self.label()),
            (true, None) => self.label().to_string(),
        }
    }

    /// Nama entri API Key di brankas OS; `None` bila penyedia ini tanpa kunci (Ollama, Claude CLI).
    /// Claude memakai nama lama supaya kunci yang sudah tersimpan tetap terbaca.
    pub fn key_name(self) -> Option<&'static str> {
        Some(match self {
            Self::ClaudeApi => "anthropic_api_key",
            Self::Gemini => "gemini_api_key",
            Self::DeepSeek => "deepseek_api_key",
            Self::Grok => "grok_api_key",
            Self::OpenAi => "openai_api_key",
            Self::OpenRouter => "openrouter_api_key",
            Self::Groq => "groq_api_key",
            Self::Ollama | Self::ClaudeCli => return None,
        })
    }

    /// Alamat `chat/completions` bergaya OpenAI dan model bawaan; `None` untuk penyedia lain.
    /// shortcut: nama model bawaan bisa usang; pengguna bisa menimpanya di Setelan, perbarui bila berganti.
    pub fn openai_compatible(self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            Self::Gemini => (
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
                "gemini-flash-latest",
            ),
            Self::DeepSeek => ("https://api.deepseek.com/chat/completions", "deepseek-chat"),
            Self::Grok => ("https://api.x.ai/v1/chat/completions", "grok-4"),
            Self::OpenAi => ("https://api.openai.com/v1/chat/completions", "gpt-4o-mini"),
            Self::OpenRouter => (
                "https://openrouter.ai/api/v1/chat/completions",
                "openrouter/auto",
            ),
            Self::Groq => (
                "https://api.groq.com/openai/v1/chat/completions",
                "llama-3.3-70b-versatile",
            ),
            Self::ClaudeApi | Self::Ollama | Self::ClaudeCli => return None,
        })
    }
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
pub const MAX_FOCUS_MINUTES: u32 = 120;
pub const MAX_BREAK_MINUTES: u32 = 60;

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
    /// Id maskot (`mino`, `kiko`, `puff`).
    pub mascot: String,
    /// Foto latar pulau sebagai data URL gambar yang sudah dikecilkan; kosong = hitam polos.
    pub background: String,
    /// Sumber latar pulau: `static` (foto di atas atau hitam), `album`, `time`, atau `wallpaper`.
    pub background_mode: String,
    /// Lama fokus dan istirahat Pomodoro (menit).
    pub focus_minutes: u32,
    pub break_minutes: u32,
    /// Sembunyikan pulau saat ada aplikasi layar penuh (game, video, presentasi).
    pub hide_on_fullscreen: bool,
    /// Bunyikan nada saat fase Pomodoro selesai.
    pub sound_enabled: bool,
    /// Cari lirik berwaktu lagu yang diputar (judul dan artis dikirim ke LRCLIB).
    pub lyrics_enabled: bool,
    /// Alamat kalender iCal (https) untuk pengingat rapat; kosong = mati.
    pub calendar_url: String,
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
            mascot: "mino".into(),
            background: String::new(),
            background_mode: "static".into(),
            focus_minutes: 25,
            break_minutes: 5,
            hide_on_fullscreen: true,
            sound_enabled: true,
            lyrics_enabled: true,
            calendar_url: String::new(),
        }
    }
}

pub const MAX_BACKGROUND_BYTES: usize = 400_000;
const BACKGROUND_MODES: [&str; 4] = ["static", "album", "time", "wallpaper"];

fn is_image_data_url(s: &str) -> bool {
    let Some(rest) = ["jpeg", "png", "webp"]
        .iter()
        .find_map(|t| s.strip_prefix(&format!("data:image/{t};base64,")))
    else {
        return false;
    };
    s.len() <= MAX_BACKGROUND_BYTES
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
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
        self.focus_minutes = self.focus_minutes.clamp(1, MAX_FOCUS_MINUTES);
        self.break_minutes = self.break_minutes.clamp(1, MAX_BREAK_MINUTES);
        for id in [&mut self.theme, &mut self.mascot] {
            id.retain(|c| c.is_ascii_alphanumeric() || c == '-');
            id.truncate(32);
        }
        if !BACKGROUND_MODES.contains(&self.background_mode.as_str()) {
            self.background_mode = "static".into();
        }
        if !is_image_data_url(&self.background) {
            self.background.clear();
        }
        self.calendar_url = self.calendar_url.trim().to_string();
        if !is_public_https_url(&self.calendar_url) {
            self.calendar_url.clear();
        }
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
    fn mode_latar_hanya_dari_daftar_yang_dikenal() {
        for ok in ["static", "album", "time", "wallpaper"] {
            let s = Settings {
                background_mode: ok.into(),
                ..Default::default()
            }
            .sanitized();
            assert_eq!(s.background_mode, ok);
        }
        let s = Settings {
            background_mode: "javascript:1".into(),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.background_mode, "static");
    }

    #[test]
    fn sanitized_hanya_menerima_latar_data_url_gambar_berukuran_wajar() {
        let ok = "data:image/jpeg;base64,/9j/4AAQ==";
        let s = Settings {
            background: ok.into(),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.background, ok);
        for bad in [
            "https://x.test/a.png",
            "data:text/html;base64,PHNjcmlwdD4=",
            "data:image/svg+xml;base64,PHN2Zz4=",
            "data:image/png;base64,AA\")}body{x:url(",
        ] {
            let s = Settings {
                background: bad.into(),
                ..Default::default()
            }
            .sanitized();
            assert!(s.background.is_empty(), "{bad}");
        }
        let big = format!("data:image/png;base64,{}", "A".repeat(MAX_BACKGROUND_BYTES));
        let s = Settings {
            background: big,
            ..Default::default()
        }
        .sanitized();
        assert!(s.background.is_empty());
    }

    #[test]
    fn sanitized_membatasi_durasi_pomodoro() {
        let s = Settings {
            focus_minutes: 0,
            break_minutes: 999,
            ..Default::default()
        }
        .sanitized();
        assert_eq!((s.focus_minutes, s.break_minutes), (1, MAX_BREAK_MINUTES));
    }

    #[test]
    fn json_lama_tanpa_field_baru_memakai_default() {
        let s: Settings = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert_eq!(s.language, Language::En);
        assert_eq!(s.display_seconds, 8);
    }
}
