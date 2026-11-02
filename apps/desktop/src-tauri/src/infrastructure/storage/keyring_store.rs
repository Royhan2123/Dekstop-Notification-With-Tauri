use keyring::{Entry, Error};

use crate::domain::error::SettingsError;
use crate::domain::ports::SecretStore;

/// Brankas OS lewat crate `keyring`. Nilai rahasia tidak pernah masuk log atau pesan error.
pub struct KeyringStore {
    service: String,
}

impl KeyringStore {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }

    fn entry(&self, key: &str) -> Result<Entry, SettingsError> {
        Entry::new(&self.service, key).map_err(fail)
    }
}

fn fail(e: Error) -> SettingsError {
    SettingsError::Storage(e.to_string())
}

impl SecretStore for KeyringStore {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(fail(e)),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError> {
        self.entry(key)?.set_password(value).map_err(fail)
    }

    fn delete(&self, key: &str) -> Result<(), SettingsError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(e) => Err(fail(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Menyentuh brankas OS sungguhan: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn simpan_baca_hapus_di_brankas_os() {
        let s = KeyringStore::new("isla-desk-test");
        s.delete("probe").unwrap();
        assert_eq!(s.get("probe").unwrap(), None);
        s.set("probe", "rahasia").unwrap();
        assert_eq!(s.get("probe").unwrap().as_deref(), Some("rahasia"));
        s.delete("probe").unwrap();
        assert_eq!(s.get("probe").unwrap(), None);
    }
}
