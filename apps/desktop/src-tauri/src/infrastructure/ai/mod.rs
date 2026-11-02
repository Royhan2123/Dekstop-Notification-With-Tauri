//! Adapter AI: Claude API, Ollama lokal, dan Claude CLI. Semuanya hanya `complete(system, user)`.
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};

use crate::domain::error::AiError;
use crate::domain::model::settings::AiProvider;
use crate::domain::ports::{AiClient, AiProviderFactory, SecretStore};

mod claude_cli;

pub use claude_cli::ClaudeCli;

const ANTHROPIC_KEY: &str = "anthropic_api_key";
const DEFAULT_CLAUDE_MODEL: &str = "claude-sonnet-5-5";
const OLLAMA_URL: &str = "http://127.0.0.1:11434/api/chat";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
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
    let res = req
        .json(body)
        .send()
        .map_err(|e| AiError::Unavailable(format!("{what}: {e}")))?;
    let status = res.status();
    if !status.is_success() {
        return Err(AiError::Unavailable(format!("{what}: HTTP {status}")));
    }
    res.json().map_err(|_| AiError::BadResponse)
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
        text_at(&post(req, &body, "Claude API")?, "/content/0/text")
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
}

pub struct DefaultAiFactory {
    secrets: Arc<dyn SecretStore>,
}

impl DefaultAiFactory {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self { secrets }
    }
}

impl AiProviderFactory for DefaultAiFactory {
    fn build(&self, provider: AiProvider, model: &str) -> Result<Arc<dyn AiClient>, AiError> {
        let model = model.trim();
        match provider {
            AiProvider::ClaudeApi => {
                let key = self
                    .secrets
                    .get(ANTHROPIC_KEY)
                    .map_err(|e| AiError::Unavailable(e.to_string()))?
                    .ok_or_else(|| {
                        AiError::NotConfigured("API Key Claude belum diisi di Setelan".into())
                    })?;
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
}
