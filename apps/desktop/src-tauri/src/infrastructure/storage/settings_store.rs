use std::fs;
use std::path::PathBuf;

use crate::domain::error::SettingsError;
use crate::domain::model::settings::Settings;
use crate::domain::ports::SettingsRepository;

/// Setelan sebagai satu file JSON di folder data aplikasi.
/// (Pengganti `tauri-plugin-store` dari doc: format sama, tanpa ketergantungan Tauri, bisa dites.)
pub struct JsonSettingsStore {
    path: PathBuf,
}

impl JsonSettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SettingsRepository for JsonSettingsStore {
    fn load(&self) -> Settings {
        let text = match fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(_) => return Settings::default(),
        };
        match serde_json::from_str::<Settings>(&text) {
            Ok(s) => s.sanitized(),
            Err(e) => {
                // File rusak: simpan salinannya agar tidak hilang, lalu mulai dari default.
                tracing::warn!("setelan rusak, memakai default: {e}");
                let _ = fs::rename(&self.path, self.path.with_extension("json.bak"));
                Settings::default()
            }
        }
    }

    fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let fail = |e: &dyn std::fmt::Display| SettingsError::Storage(e.to_string());
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| fail(&e))?;
        }
        let json = serde_json::to_string_pretty(settings).map_err(|e| fail(&e))?;
        // Tulis ke file sementara lalu ganti, supaya crash tidak menyisakan file setengah jadi.
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| fail(&e))?;
        fs::rename(&tmp, &self.path).map_err(|e| fail(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::settings::Language;

    fn store(dir: &tempfile::TempDir) -> JsonSettingsStore {
        JsonSettingsStore::new(dir.path().join("sub").join("settings.json"))
    }

    #[test]
    fn belum_ada_file_memberi_default_dan_simpan_lalu_muat_kembali() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir);
        assert_eq!(s.load(), Settings::default());
        let saved = Settings {
            language: Language::En,
            onboarded: true,
            ..Default::default()
        };
        s.save(&saved).unwrap();
        assert_eq!(s.load(), saved);
    }

    #[test]
    fn file_rusak_dicadangkan_dan_default_dipakai() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir);
        s.save(&Settings::default()).unwrap();
        fs::write(&s.path, "{bukan json").unwrap();
        assert_eq!(s.load(), Settings::default());
        assert!(s.path.with_extension("json.bak").exists());
    }
}
