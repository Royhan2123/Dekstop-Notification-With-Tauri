//! Perlengkapan tes bersama untuk service yang butuh `SettingsService` dan `IslandService` nyata.
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use super::island_service::IslandService;
use super::settings_service::SettingsService;
use crate::domain::error::SettingsError;
use crate::domain::island::state::IslandState;
use crate::domain::model::settings::Settings;
use crate::domain::ports::{
    Autostart, SecretStore, SettingsRepository, SettingsSink, SnapshotSink,
};

pub struct NullSink;
impl SnapshotSink for NullSink {
    fn publish(&self, _: &IslandState) {}
}
impl SettingsSink for NullSink {
    fn publish(&self, _: &Settings) {}
}

pub struct MemRepo(pub Mutex<Settings>);
impl SettingsRepository for MemRepo {
    fn load(&self) -> Settings {
        self.0.lock().unwrap().clone()
    }
    fn save(&self, s: &Settings) -> Result<(), SettingsError> {
        *self.0.lock().unwrap() = s.clone();
        Ok(())
    }
}

pub struct NoSecrets;
impl SecretStore for NoSecrets {
    fn get(&self, _: &str) -> Result<Option<String>, SettingsError> {
        Ok(None)
    }
    fn set(&self, _: &str, _: &str) -> Result<(), SettingsError> {
        Ok(())
    }
    fn delete(&self, _: &str) -> Result<(), SettingsError> {
        Ok(())
    }
}

pub struct NoAutostart;
impl Autostart for NoAutostart {
    fn set(&self, _: bool) -> Result<(), SettingsError> {
        Ok(())
    }
}

/// Pulau dan setelan nyata di atas penyimpanan memori; `Runtime` harus dipegang selama tes.
pub fn services(initial: Settings) -> (Arc<SettingsService>, Arc<IslandService>, Runtime) {
    let rt = Runtime::new().unwrap();
    let island = IslandService::new(Arc::new(NullSink), rt.handle().clone());
    let settings = SettingsService::new(
        Arc::new(MemRepo(Mutex::new(initial))),
        Arc::new(NoSecrets),
        Arc::new(NoAutostart),
        Arc::new(NullSink),
        island.clone(),
    );
    (settings, island, rt)
}

/// Brankas rahasia di memori untuk tes.
#[derive(Default)]
pub struct MemSecrets(pub Mutex<std::collections::HashMap<String, String>>);
impl SecretStore for MemSecrets {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError> {
        self.0.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    fn delete(&self, key: &str) -> Result<(), SettingsError> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}
