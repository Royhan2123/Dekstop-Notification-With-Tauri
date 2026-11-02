use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::bridge::{BridgeCommand, clamp_rate, clean_query, is_extension_origin};
use crate::domain::error::{BridgeError, SettingsError};
use crate::domain::island::action::IslandAction;
use crate::domain::model::settings::Language;
use crate::domain::ports::{BridgeHandler, BrowserBridge, SecretStore};

const TOKEN_KEY: &str = "extension_token";
const ORIGIN_KEY: &str = "extension_origin";
const CODE_TTL: Duration = Duration::from_secs(300);
const MAX_ATTEMPTS: u8 = 5;

struct Pending {
    code: String,
    expires: Instant,
    attempts: u8,
}

/// Pairing, otorisasi, dan perintah ke ekstensi Isla Link.
/// Token dan asal ekstensi disimpan di brankas OS; kode pairing hanya di memori.
pub struct BridgeService {
    bridge: Arc<dyn BrowserBridge>,
    secrets: Arc<dyn SecretStore>,
    island: Arc<IslandService>,
    settings: Arc<SettingsService>,
    pending: Mutex<Option<Pending>>,
}

fn random_hex() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn same(a: &str, b: &str) -> bool {
    // Bandingkan tanpa berhenti di selisih pertama agar waktu tidak membocorkan token.
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (x, y)| acc | (x ^ y))
            == 0
}

impl BridgeService {
    pub fn new(
        bridge: Arc<dyn BrowserBridge>,
        secrets: Arc<dyn SecretStore>,
        island: Arc<IslandService>,
        settings: Arc<SettingsService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            bridge,
            secrets,
            island,
            settings,
            pending: Mutex::new(None),
        })
    }

    pub fn is_paired(&self) -> Result<bool, SettingsError> {
        Ok(self.secrets.get(TOKEN_KEY)?.is_some())
    }

    /// Buat kode 6 digit baru (berlaku 5 menit, sekali pakai) dan tampilkan di pulau.
    pub fn begin_pairing(&self) -> String {
        self.begin_pairing_at(Instant::now())
    }

    fn begin_pairing_at(&self, now: Instant) -> String {
        let n = u32::from_le_bytes(
            uuid::Uuid::new_v4().as_bytes()[..4]
                .try_into()
                .unwrap_or_default(),
        );
        let code = format!("{:06}", n % 1_000_000);
        *self.pending() = Some(Pending {
            code: code.clone(),
            expires: now + CODE_TTL,
            attempts: 0,
        });
        let text = match self.settings.get().language {
            Language::Id => format!("Kode Isla Link: {code}"),
            Language::En => format!("Isla Link code: {code}"),
        };
        self.island.dispatch(IslandAction::Notice(text));
        code
    }

    /// Cabut pairing: hapus token dan putuskan ekstensi yang sedang tersambung.
    pub fn unpair(&self) -> Result<(), SettingsError> {
        self.secrets.delete(TOKEN_KEY)?;
        self.secrets.delete(ORIGIN_KEY)?;
        *self.pending() = None;
        self.bridge.disconnect();
        Ok(())
    }

    pub fn set_rate(&self, rate: f64) -> Result<(), BridgeError> {
        let rate = clamp_rate(rate).ok_or(BridgeError::Invalid)?;
        self.bridge.send(&BridgeCommand::SetRate(rate))
    }

    pub fn search(&self, query: &str) -> Result<(), BridgeError> {
        let q = clean_query(query).ok_or(BridgeError::Invalid)?;
        self.bridge.send(&BridgeCommand::Search(q))
    }

    fn pair_at(&self, origin: &str, code: &str, now: Instant) -> Option<String> {
        if !is_extension_origin(origin) {
            return None;
        }
        {
            let mut guard = self.pending();
            let p = guard.as_mut()?;
            if now >= p.expires || p.attempts >= MAX_ATTEMPTS {
                *guard = None;
                return None;
            }
            if !same(&p.code, code.trim()) {
                p.attempts += 1;
                return None;
            }
            *guard = None;
        }
        let token = format!("{}{}", random_hex(), random_hex());
        self.secrets.set(TOKEN_KEY, &token).ok()?;
        self.secrets.set(ORIGIN_KEY, origin).ok()?;
        Some(token)
    }

    fn pending(&self) -> std::sync::MutexGuard<'_, Option<Pending>> {
        self.pending.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl BridgeHandler for BridgeService {
    fn pair(&self, origin: &str, code: &str) -> Option<String> {
        self.pair_at(origin, code, Instant::now())
    }

    fn authenticate(&self, origin: &str, token: &str) -> bool {
        let (Ok(Some(t)), Ok(Some(o))) =
            (self.secrets.get(TOKEN_KEY), self.secrets.get(ORIGIN_KEY))
        else {
            return false;
        };
        is_extension_origin(origin) && same(&o, origin) && same(&t, token)
    }

    fn connection_changed(&self, connected: bool) {
        self.island
            .dispatch(IslandAction::ExtensionConnected(connected));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{MemSecrets, services};
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::settings::Settings;

    #[derive(Default)]
    struct FakeBridge {
        sent: Mutex<Vec<BridgeCommand>>,
        connected: Mutex<bool>,
        disconnects: Mutex<u32>,
    }
    impl BrowserBridge for FakeBridge {
        fn send(&self, cmd: &BridgeCommand) -> Result<(), BridgeError> {
            if !*self.connected.lock().unwrap() {
                return Err(BridgeError::NotConnected);
            }
            self.sent.lock().unwrap().push(cmd.clone());
            Ok(())
        }
        fn disconnect(&self) {
            *self.disconnects.lock().unwrap() += 1;
        }
    }

    struct Fx {
        svc: Arc<BridgeService>,
        bridge: Arc<FakeBridge>,
        island: Arc<IslandService>,
        _rt: tokio::runtime::Runtime,
    }

    fn fx() -> Fx {
        let (settings, island, rt) = services(Settings::default());
        let bridge = Arc::new(FakeBridge::default());
        let svc = BridgeService::new(
            bridge.clone(),
            Arc::new(MemSecrets::default()),
            island.clone(),
            settings,
        );
        Fx {
            svc,
            bridge,
            island,
            _rt: rt,
        }
    }

    const ORIGIN: &str = "chrome-extension://abcdef";

    #[test]
    fn kode_benar_menghasilkan_token_dan_hanya_sekali_pakai() {
        let f = fx();
        let code = f.svc.begin_pairing();
        assert_eq!(code.len(), 6);
        assert!(matches!(
            f.island.snapshot().view,
            View::Expanded(Content::Notice(t)) if t.contains(&code)
        ));
        assert!(!f.svc.is_paired().unwrap());
        let token = f.svc.pair(ORIGIN, &code).unwrap();
        assert!(f.svc.is_paired().unwrap());
        assert!(f.svc.authenticate(ORIGIN, &token));
        assert_eq!(f.svc.pair(ORIGIN, &code), None, "kode sudah terpakai");
    }

    #[test]
    fn asal_atau_token_salah_ditolak_dan_asal_terkunci_pada_ekstensi_pertama() {
        let f = fx();
        let code = f.svc.begin_pairing();
        assert_eq!(f.svc.pair("https://evil.example", &code), None);
        let token = f.svc.pair(ORIGIN, &code).unwrap();
        assert!(!f.svc.authenticate("chrome-extension://lain", &token));
        assert!(!f.svc.authenticate(ORIGIN, "salah"));
        assert!(!f.svc.authenticate("https://evil.example", &token));
    }

    #[test]
    fn kode_kedaluwarsa_atau_terlalu_banyak_salah_hangus() {
        let f = fx();
        let t0 = Instant::now();
        let code = f.svc.begin_pairing_at(t0);
        assert_eq!(f.svc.pair_at(ORIGIN, &code, t0 + CODE_TTL), None);
        assert_eq!(
            f.svc.pair_at(ORIGIN, &code, t0),
            None,
            "hangus setelah kedaluwarsa"
        );

        let code = f.svc.begin_pairing_at(t0);
        let wrong = if code == "000000" { "111111" } else { "000000" };
        for _ in 0..MAX_ATTEMPTS {
            assert_eq!(f.svc.pair_at(ORIGIN, wrong, t0), None);
        }
        assert_eq!(
            f.svc.pair_at(ORIGIN, &code, t0),
            None,
            "kode benar pun ditolak setelah 5 kali salah"
        );
    }

    #[test]
    fn perintah_divalidasi_dan_butuh_ekstensi_tersambung() {
        let f = fx();
        assert!(matches!(
            f.svc.set_rate(1.5),
            Err(BridgeError::NotConnected)
        ));
        *f.bridge.connected.lock().unwrap() = true;
        f.svc.set_rate(9.0).unwrap();
        f.svc.search("  lofi  ").unwrap();
        assert!(matches!(
            f.svc.set_rate(f64::NAN),
            Err(BridgeError::Invalid)
        ));
        assert!(matches!(f.svc.search("  "), Err(BridgeError::Invalid)));
        assert_eq!(
            *f.bridge.sent.lock().unwrap(),
            vec![
                BridgeCommand::SetRate(2.0),
                BridgeCommand::Search("lofi".into())
            ]
        );
    }

    #[test]
    fn unpair_menghapus_token_memutus_ekstensi_dan_status_mengikuti_koneksi() {
        let f = fx();
        let code = f.svc.begin_pairing();
        let token = f.svc.pair(ORIGIN, &code).unwrap();
        f.svc.unpair().unwrap();
        assert!(!f.svc.is_paired().unwrap());
        assert!(!f.svc.authenticate(ORIGIN, &token));
        assert_eq!(*f.bridge.disconnects.lock().unwrap(), 1);
        f.svc.connection_changed(true);
        assert!(f.island.snapshot().extension);
        f.svc.connection_changed(false);
        assert!(!f.island.snapshot().extension);
    }
}
