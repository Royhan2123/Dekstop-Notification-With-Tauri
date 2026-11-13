use std::sync::{Arc, Mutex, MutexGuard};

use crate::application::island_service::IslandService;
use crate::domain::error::SettingsError;
use crate::domain::model::settings::{AiProvider, Settings};
use crate::domain::ports::{Autostart, SecretStore, SettingsRepository, SettingsSink};

const MAX_KEY_LEN: usize = 512;

/// Memegang setelan aktif, menyimpannya, dan menerapkannya ke bagian aplikasi lain.
/// API Key hanya lewat `SecretStore`: tidak ada di `Settings` dan tidak pernah dikembalikan.
pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
    secrets: Arc<dyn SecretStore>,
    autostart: Arc<dyn Autostart>,
    sink: Arc<dyn SettingsSink>,
    island: Arc<IslandService>,
    current: Mutex<Settings>,
}

impl SettingsService {
    pub fn new(
        repo: Arc<dyn SettingsRepository>,
        secrets: Arc<dyn SecretStore>,
        autostart: Arc<dyn Autostart>,
        sink: Arc<dyn SettingsSink>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        let current = repo.load();
        island.set_auto_dismiss_ms(u64::from(current.display_seconds) * 1000);
        Arc::new(Self {
            repo,
            secrets,
            autostart,
            sink,
            island,
            current: Mutex::new(current),
        })
    }

    pub fn get(&self) -> Settings {
        self.lock().clone()
    }

    /// Validasi, simpan, lalu terapkan. Bila gagal disimpan, setelan lama tetap berlaku.
    pub fn update(&self, next: Settings) -> Result<Settings, SettingsError> {
        let next = next.sanitized();
        let mut current = self.lock();
        if next == *current {
            return Ok(next);
        }
        if next.autostart != current.autostart {
            self.autostart.set(next.autostart)?;
        }
        self.repo.save(&next)?;
        self.island
            .set_auto_dismiss_ms(u64::from(next.display_seconds) * 1000);
        *current = next.clone();
        self.sink.publish(&next);
        Ok(next)
    }

    pub fn has_api_key(&self, provider: AiProvider) -> Result<bool, SettingsError> {
        let Some(name) = provider.key_name() else {
            return Ok(false);
        };
        Ok(self.secrets.get(name)?.is_some())
    }

    /// Penyedia yang siap dipakai: yang kuncinya sudah tersimpan, ditambah yang tanpa kunci
    /// (Ollama, Claude Code); kedua yang terakhir tidak diperiksa apakah sedang berjalan.
    pub fn ready_providers(&self) -> Result<Vec<AiProvider>, SettingsError> {
        let mut ready = Vec::new();
        for p in AiProvider::ALL {
            if p.key_name().is_none() || self.has_api_key(p)? {
                ready.push(p);
            }
        }
        Ok(ready)
    }

    pub fn set_api_key(&self, provider: AiProvider, key: &str) -> Result<(), SettingsError> {
        let key = key.trim();
        let name = provider.key_name().ok_or(SettingsError::InvalidKey)?;
        if key.is_empty() || key.len() > MAX_KEY_LEN || key.chars().any(char::is_control) {
            return Err(SettingsError::InvalidKey);
        }
        self.secrets.set(name, key)
    }

    pub fn clear_api_key(&self, provider: AiProvider) -> Result<(), SettingsError> {
        match provider.key_name() {
            Some(name) => self.secrets.delete(name),
            None => Ok(()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.current.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::island::state::IslandState;
    use crate::domain::model::settings::Language;
    use crate::domain::ports::SnapshotSink;
    use std::collections::HashMap;
    use tokio::runtime::Handle;

    struct NullSink;
    impl SnapshotSink for NullSink {
        fn publish(&self, _: &IslandState) {}
    }

    #[derive(Default)]
    struct MemRepo {
        saved: Mutex<Option<Settings>>,
        fail: Mutex<bool>,
    }
    impl SettingsRepository for Arc<MemRepo> {
        fn load(&self) -> Settings {
            self.saved.lock().unwrap().clone().unwrap_or_default()
        }
        fn save(&self, s: &Settings) -> Result<(), SettingsError> {
            if *self.fail.lock().unwrap() {
                return Err(SettingsError::Storage("disk penuh".into()));
            }
            *self.saved.lock().unwrap() = Some(s.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemSecrets(Mutex<HashMap<String, String>>);
    impl SecretStore for Arc<MemSecrets> {
        fn get(&self, k: &str) -> Result<Option<String>, SettingsError> {
            Ok(self.0.lock().unwrap().get(k).cloned())
        }
        fn set(&self, k: &str, v: &str) -> Result<(), SettingsError> {
            self.0.lock().unwrap().insert(k.into(), v.into());
            Ok(())
        }
        fn delete(&self, k: &str) -> Result<(), SettingsError> {
            self.0.lock().unwrap().remove(k);
            Ok(())
        }
    }

    #[derive(Default)]
    struct CountingAutostart(Mutex<Vec<bool>>);
    impl Autostart for Arc<CountingAutostart> {
        fn set(&self, e: bool) -> Result<(), SettingsError> {
            self.0.lock().unwrap().push(e);
            Ok(())
        }
    }

    #[derive(Default)]
    struct CountingSink(Mutex<usize>);
    impl SettingsSink for Arc<CountingSink> {
        fn publish(&self, _: &Settings) {
            *self.0.lock().unwrap() += 1;
        }
    }

    struct Fixture {
        svc: Arc<SettingsService>,
        repo: Arc<MemRepo>,
        secrets: Arc<MemSecrets>,
        autostart: Arc<CountingAutostart>,
        published: Arc<CountingSink>,
    }

    fn fixture() -> Fixture {
        let repo = Arc::new(MemRepo::default());
        let secrets = Arc::new(MemSecrets::default());
        let autostart = Arc::new(CountingAutostart::default());
        let published = Arc::new(CountingSink::default());
        let island = IslandService::new(Arc::new(NullSink), Handle::current());
        let svc = SettingsService::new(
            Arc::new(repo.clone()),
            Arc::new(secrets.clone()),
            Arc::new(autostart.clone()),
            Arc::new(published.clone()),
            island,
        );
        Fixture {
            svc,
            repo,
            secrets,
            autostart,
            published,
        }
    }

    #[tokio::test]
    async fn update_merapikan_menyimpan_dan_mempublikasikan_sekali() {
        let f = fixture();
        let out = f
            .svc
            .update(Settings {
                language: Language::En,
                display_seconds: 1,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(out.display_seconds, 2);
        assert_eq!(f.svc.get(), out);
        assert_eq!(f.repo.saved.lock().unwrap().as_ref(), Some(&out));
        f.svc.update(out).unwrap(); // sama persis: tidak menyimpan/mempublikasikan lagi
        assert_eq!(*f.published.0.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn autostart_hanya_dipanggil_saat_berubah() {
        let f = fixture();
        let base = f.svc.get();
        f.svc
            .update(Settings {
                reduce_motion: true,
                ..base.clone()
            })
            .unwrap();
        assert!(f.autostart.0.lock().unwrap().is_empty());
        f.svc
            .update(Settings {
                autostart: true,
                ..base
            })
            .unwrap();
        assert_eq!(*f.autostart.0.lock().unwrap(), vec![true]);
    }

    #[tokio::test]
    async fn gagal_simpan_tidak_mengubah_setelan_aktif() {
        let f = fixture();
        *f.repo.fail.lock().unwrap() = true;
        let before = f.svc.get();
        let res = f.svc.update(Settings {
            reduce_motion: true,
            ..before.clone()
        });
        assert!(matches!(res, Err(SettingsError::Storage(_))));
        assert_eq!(f.svc.get(), before);
        assert_eq!(*f.published.0.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn api_key_divalidasi_disimpan_di_brankas_dan_bisa_dihapus() {
        const CLAUDE: AiProvider = AiProvider::ClaudeApi;
        let f = fixture();
        assert!(!f.svc.has_api_key(CLAUDE).unwrap());
        for bad in ["", "   ", "a\nb", &"k".repeat(MAX_KEY_LEN + 1)] {
            assert!(matches!(
                f.svc.set_api_key(CLAUDE, bad),
                Err(SettingsError::InvalidKey)
            ));
        }
        f.svc.set_api_key(CLAUDE, "  sk-ant-contoh  ").unwrap();
        assert!(f.svc.has_api_key(CLAUDE).unwrap());
        assert_eq!(
            f.secrets
                .0
                .lock()
                .unwrap()
                .get("anthropic_api_key")
                .map(String::as_str),
            Some("sk-ant-contoh")
        );
        // kunci tidak ikut ke berkas setelan
        let json = serde_json::to_string(&f.svc.get()).unwrap();
        assert!(!json.contains("sk-ant"));
        f.svc.clear_api_key(CLAUDE).unwrap();
        assert!(!f.svc.has_api_key(CLAUDE).unwrap());
    }

    #[tokio::test]
    async fn penyedia_siap_adalah_yang_berkunci_ditambah_yang_tanpa_kunci() {
        let f = fixture();
        assert_eq!(
            f.svc.ready_providers().unwrap(),
            [
                AiProvider::Ollama,
                AiProvider::ClaudeCli,
                AiProvider::Jarvis
            ]
        );
        f.svc
            .set_api_key(AiProvider::Gemini, "AIza-contoh")
            .unwrap();
        f.svc
            .set_api_key(AiProvider::ClaudeApi, "sk-ant-contoh")
            .unwrap();
        assert_eq!(
            f.svc.ready_providers().unwrap(),
            [
                AiProvider::ClaudeApi,
                AiProvider::Gemini,
                AiProvider::Ollama,
                AiProvider::ClaudeCli,
                AiProvider::Jarvis
            ]
        );
    }

    #[tokio::test]
    async fn kunci_tiap_penyedia_terpisah_dan_penyedia_tanpa_kunci_ditolak() {
        let f = fixture();
        f.svc
            .set_api_key(AiProvider::Gemini, "AIza-contoh")
            .unwrap();
        assert!(f.svc.has_api_key(AiProvider::Gemini).unwrap());
        assert!(!f.svc.has_api_key(AiProvider::DeepSeek).unwrap());
        assert!(!f.svc.has_api_key(AiProvider::ClaudeApi).unwrap());
        assert!(matches!(
            f.svc.set_api_key(AiProvider::Ollama, "x"),
            Err(SettingsError::InvalidKey)
        ));
        assert!(!f.svc.has_api_key(AiProvider::Ollama).unwrap());
        f.svc.clear_api_key(AiProvider::Gemini).unwrap();
        assert!(!f.svc.has_api_key(AiProvider::Gemini).unwrap());
    }
}
