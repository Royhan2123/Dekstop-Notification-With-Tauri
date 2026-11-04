//! Adapter AI: Claude API, penyedia bergaya OpenAI (Gemini, DeepSeek, Grok, ...), Ollama lokal, dan
//! Claude CLI. Semuanya hanya `complete(system, user)`.
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};

use crate::domain::error::AiError;
use crate::domain::model::settings::AiProvider;
use crate::domain::ports::{AiClient, AiProviderFactory, SecretStore};

mod claude_cli;

pub use claude_cli::ClaudeCli;

const DEFAULT_CLAUDE_MODEL: &str = "claude-sonnet-5-5";
const OLLAMA_URL: &str = "http://127.0.0.1:11434/api/chat";
const OLLAMA_TAGS_URL: &str = "http://127.0.0.1:11434/api/tags";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_MODELS_URL: &str = "https://api.anthropic.com/v1/models?limit=100";
const TIMEOUT: Duration = Duration::from_secs(180);

fn http() -> Result<reqwest::blocking::Client, AiError> {
    reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| AiError::Unavailable(e.to_string()))
}

fn post(
    req: reqwest::blocking::RequestBuilder,
    body: &Value,
    what: &str,
) -> Result<Value, AiError> {
    call(req.json(body), what)
}

fn call(req: reqwest::blocking::RequestBuilder, what: &str) -> Result<Value, AiError> {
    let res = req
        .send()
        .map_err(|e| AiError::Unavailable(format!("{what}: {e}")))?;
    let status = res.status();
    if !status.is_success() {
        // Isi galat (kunci salah, model tidak ada, kuota habis) membuat penyebabnya terlihat di Setelan.
        let detail = res.text().unwrap_or_default();
        return Err(AiError::Unavailable(format!(
            "{what}: HTTP {status} {}",
            error_detail(&detail)
        )));
    }
    res.json().map_err(|_| AiError::BadResponse)
}

/// Pesan galat dari badan jawaban (`error.message` bila ada), satu baris dan dipendekkan.
fn error_detail(body: &str) -> String {
    let msg = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            // Bentuk umum: {"error":{"message":..}}, {"error":".."}, atau daftar seperti Gemini.
            let e = v.get(0).unwrap_or(&v).get("error")?.clone();
            e.get("message").unwrap_or(&e).as_str().map(str::to_string)
        })
        .unwrap_or_else(|| body.to_string());
    msg.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

/// Nama model dari daftar jawaban penyedia. Hanya model percakapan: embedding, suara, dan gambar disaring.
fn model_ids(v: &Value, list: &str, field: &str) -> Vec<String> {
    const NOT_CHAT: [&str; 7] = [
        "embed",
        "whisper",
        "tts",
        "dall-e",
        "moderation",
        "image",
        "audio",
    ];
    v.pointer(list)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|m| m.get(field)?.as_str())
                .map(|id| id.strip_prefix("models/").unwrap_or(id).to_string())
                .filter(|id| !NOT_CHAT.iter().any(|w| id.to_lowercase().contains(w)))
                .collect()
        })
        .unwrap_or_default()
}

/// Teks balasan Claude: gabungan semua blok bertipe `text`. Blok lain (mis. berpikir) dilewati,
/// jadi balasan yang tidak diawali blok teks tetap terbaca.
fn claude_text(v: &Value) -> Result<String, AiError> {
    let text: String = v
        .get("content")
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text")?.as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    if text.trim().is_empty() {
        let reason = v.get("stop_reason").and_then(Value::as_str).unwrap_or("?");
        return Err(AiError::Unavailable(format!(
            "Claude API: balasan tanpa teks (stop_reason: {reason})"
        )));
    }
    Ok(text)
}

fn text_at(v: &Value, pointer: &str) -> Result<String, AiError> {
    v.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or(AiError::BadResponse)
}

pub struct AnthropicClient {
    key: String,
    model: String,
}

impl AiClient for AnthropicClient {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system,
            "messages": [{ "role": "user", "content": user }],
        });
        let req = http()?
            .post(ANTHROPIC_URL)
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01");
        claude_text(&post(req, &body, "Claude API")?)
    }

    fn models(&self) -> Result<Vec<String>, AiError> {
        let req = http()?
            .get(ANTHROPIC_MODELS_URL)
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01");
        Ok(model_ids(&call(req, "Claude API")?, "/data", "id"))
    }
}

/// Gemini, DeepSeek, Grok, OpenAI, OpenRouter, dan Groq semuanya berbicara dialek `chat/completions` OpenAI.
pub struct OpenAiClient {
    name: &'static str,
    url: &'static str,
    key: String,
    model: String,
}

impl AiClient for OpenAiClient {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });
        let req = http()?.post(self.url).bearer_auth(&self.key);
        text_at(&post(req, &body, self.name)?, "/choices/0/message/content")
    }

    fn models(&self) -> Result<Vec<String>, AiError> {
        let url = self.url.replace("/chat/completions", "/models");
        let req = http()?.get(url).bearer_auth(&self.key);
        Ok(model_ids(&call(req, self.name)?, "/data", "id"))
    }
}

pub struct OllamaClient {
    model: String,
}

impl AiClient for OllamaClient {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let body = json!({
            "model": self.model,
            "stream": false,
            "format": "json",
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });
        let req = http()?.post(OLLAMA_URL);
        text_at(&post(req, &body, "Ollama")?, "/message/content")
    }

    fn models(&self) -> Result<Vec<String>, AiError> {
        let req = http()?.get(OLLAMA_TAGS_URL);
        Ok(model_ids(&call(req, "Ollama")?, "/models", "name"))
    }
}

pub struct DefaultAiFactory {
    secrets: Arc<dyn SecretStore>,
}

impl DefaultAiFactory {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self { secrets }
    }
}

impl DefaultAiFactory {
    fn key(&self, provider: AiProvider) -> Result<String, AiError> {
        let name = provider.key_name().ok_or(AiError::BadResponse)?;
        self.secrets
            .get(name)
            .map_err(|e| AiError::Unavailable(e.to_string()))?
            .ok_or_else(|| {
                AiError::NotConfigured(format!(
                    "API Key {} belum diisi di Setelan",
                    provider_name(provider)
                ))
            })
    }
}

fn provider_name(provider: AiProvider) -> &'static str {
    match provider {
        AiProvider::ClaudeApi => "Claude",
        AiProvider::Gemini => "Gemini",
        AiProvider::DeepSeek => "DeepSeek",
        AiProvider::Grok => "Grok",
        AiProvider::OpenAi => "OpenAI",
        AiProvider::OpenRouter => "OpenRouter",
        AiProvider::Groq => "Groq",
        AiProvider::Ollama => "Ollama",
        AiProvider::ClaudeCli => "Claude CLI",
    }
}

impl AiProviderFactory for DefaultAiFactory {
    fn build(&self, provider: AiProvider, model: &str) -> Result<Arc<dyn AiClient>, AiError> {
        let model = model.trim();
        match provider {
            AiProvider::ClaudeApi => {
                let key = self.key(provider)?;
                let model = if model.is_empty() {
                    DEFAULT_CLAUDE_MODEL
                } else {
                    model
                };
                Ok(Arc::new(AnthropicClient {
                    key,
                    model: model.into(),
                }))
            }
            AiProvider::Ollama if model.is_empty() => Err(AiError::NotConfigured(
                "Nama model Ollama belum diisi di Setelan".into(),
            )),
            AiProvider::Ollama => Ok(Arc::new(OllamaClient {
                model: model.into(),
            })),
            AiProvider::ClaudeCli => Ok(Arc::new(ClaudeCli::new(model))),
            other => {
                let (url, default_model) = other.openai_compatible().ok_or(AiError::BadResponse)?;
                Ok(Arc::new(OpenAiClient {
                    name: provider_name(other),
                    url,
                    key: self.key(other)?,
                    model: if model.is_empty() {
                        default_model
                    } else {
                        model
                    }
                    .into(),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::SettingsError;

    struct Secrets(Option<String>);
    impl SecretStore for Secrets {
        fn get(&self, _: &str) -> Result<Option<String>, SettingsError> {
            Ok(self.0.clone())
        }
        fn set(&self, _: &str, _: &str) -> Result<(), SettingsError> {
            Ok(())
        }
        fn delete(&self, _: &str) -> Result<(), SettingsError> {
            Ok(())
        }
    }

    #[test]
    fn jawaban_dibaca_dari_lokasi_yang_benar_atau_dianggap_rusak() {
        let v = json!({ "content": [{ "type": "text", "text": "halo" }] });
        assert_eq!(text_at(&v, "/content/0/text").unwrap(), "halo");
        assert!(matches!(
            text_at(&json!({}), "/content/0/text"),
            Err(AiError::BadResponse)
        ));
    }

    #[test]
    fn factory_menolak_konfigurasi_yang_belum_lengkap() {
        let tanpa_kunci = DefaultAiFactory::new(Arc::new(Secrets(None)));
        assert!(matches!(
            tanpa_kunci.build(AiProvider::ClaudeApi, "").err(),
            Some(AiError::NotConfigured(_))
        ));
        assert!(matches!(
            tanpa_kunci.build(AiProvider::Ollama, "  ").err(),
            Some(AiError::NotConfigured(_))
        ));
        assert!(tanpa_kunci.build(AiProvider::Ollama, "llama3").is_ok());
        assert!(tanpa_kunci.build(AiProvider::ClaudeCli, "").is_ok());
        let dengan_kunci = DefaultAiFactory::new(Arc::new(Secrets(Some("k".into()))));
        assert!(dengan_kunci.build(AiProvider::ClaudeApi, "").is_ok());
    }

    #[test]
    fn semua_penyedia_bergaya_openai_butuh_kunci_dan_punya_alamat_https() {
        let tanpa_kunci = DefaultAiFactory::new(Arc::new(Secrets(None)));
        let dengan_kunci = DefaultAiFactory::new(Arc::new(Secrets(Some("k".into()))));
        for p in [
            AiProvider::Gemini,
            AiProvider::DeepSeek,
            AiProvider::Grok,
            AiProvider::OpenAi,
            AiProvider::OpenRouter,
            AiProvider::Groq,
        ] {
            let (url, model) = p.openai_compatible().unwrap();
            assert!(url.starts_with("https://") && !model.is_empty(), "{p:?}");
            assert!(p.key_name().is_some(), "{p:?}");
            assert!(
                matches!(tanpa_kunci.build(p, "").err(), Some(AiError::NotConfigured(m)) if !m.is_empty()),
                "{p:?}"
            );
            assert!(dengan_kunci.build(p, "").is_ok(), "{p:?}");
        }
        assert!(AiProvider::Ollama.key_name().is_none());
        assert!(AiProvider::ClaudeApi.openai_compatible().is_none());
    }

    #[test]
    fn balasan_claude_dibaca_dari_blok_teks_di_mana_pun() {
        let biasa = json!({ "content": [{ "type": "text", "text": "halo" }] });
        assert_eq!(claude_text(&biasa).unwrap(), "halo");
        let berpikir = json!({ "content": [
            { "type": "thinking", "thinking": "hmm" },
            { "type": "text", "text": "ha" },
            { "type": "text", "text": "lo" }
        ]});
        assert_eq!(claude_text(&berpikir).unwrap(), "halo");
        let kosong = json!({ "content": [], "stop_reason": "max_tokens" });
        let err = claude_text(&kosong).unwrap_err().to_string();
        assert!(err.contains("max_tokens"), "{err}");
    }

    #[test]
    fn daftar_model_hanya_model_percakapan_tanpa_awalan_models() {
        let gemini = json!({ "data": [
            { "id": "models/gemini-2.5-flash" },
            { "id": "models/text-embedding-004" },
            { "id": "models/gemini-2.5-pro" },
            {}
        ]});
        assert_eq!(
            model_ids(&gemini, "/data", "id"),
            ["gemini-2.5-flash", "gemini-2.5-pro"]
        );
        let ollama =
            json!({ "models": [{ "name": "llama3:latest" }, { "name": "nomic-embed-text" }] });
        assert_eq!(model_ids(&ollama, "/models", "name"), ["llama3:latest"]);
        assert!(model_ids(&json!({}), "/data", "id").is_empty());
    }

    #[test]
    fn pesan_galat_penyedia_diambil_ringkas() {
        assert_eq!(
            error_detail(r#"{"error":{"message":"model tidak ada","code":404}}"#),
            "model tidak ada"
        );
        assert_eq!(error_detail(r#"{"error":"kunci salah"}"#), "kunci salah");
        assert_eq!(
            error_detail(r#"[{"error":{"message":"API key not valid"}}]"#),
            "API key not valid"
        );
        assert_eq!(
            error_detail("<html>\n  gagal\n</html>"),
            "<html> gagal </html>"
        );
        assert_eq!(error_detail(&"x".repeat(900)).len(), 200);
    }
}
