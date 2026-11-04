use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::application::island_service::IslandService;
use crate::domain::error::GithubError;
use crate::domain::github::{Radar, news};
use crate::domain::island::action::IslandAction;
use crate::domain::ports::{GithubApi, SecretStore};

const TOKEN_KEY: &str = "github_token";
const MAX_TOKEN_LEN: usize = 255;
/// Hasil terakhir dipakai ulang selama ini supaya membuka Setelan berkali-kali tidak menghabiskan kuota.
const FRESH: Duration = Duration::from_secs(60);
/// Pulau hanya menampilkan sebanyak ini kabar sekali jalan; sisanya cukup terlihat di Setelan.
const MAX_NOTICES: usize = 2;

struct Last {
    at: Instant,
    radar: Radar,
}

/// Radar GitHub: PR yang menunggu review kita dan status CI PR kita. Token di brankas OS.
pub struct GithubService {
    api: Arc<dyn GithubApi>,
    secrets: Arc<dyn SecretStore>,
    island: Arc<IslandService>,
    last: Mutex<Option<Last>>,
}

impl GithubService {
    pub fn new(
        api: Arc<dyn GithubApi>,
        secrets: Arc<dyn SecretStore>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            api,
            secrets,
            island,
            last: Mutex::new(None),
        })
    }

    pub fn has_token(&self) -> Result<bool, GithubError> {
        Ok(self.secrets.get(TOKEN_KEY)?.is_some())
    }

    /// Token hanya masuk ke brankas OS; tidak ada jalan untuk membacanya kembali.
    pub fn set_token(&self, token: &str) -> Result<(), GithubError> {
        let token = token.trim();
        if token.is_empty() || token.len() > MAX_TOKEN_LEN || token.chars().any(char::is_control) {
            return Err(GithubError::Invalid("token tidak valid".into()));
        }
        self.secrets.set(TOKEN_KEY, token)?;
        *self.last.lock().unwrap_or_else(|e| e.into_inner()) = None;
        Ok(())
    }

    pub fn clear_token(&self) -> Result<(), GithubError> {
        self.secrets.delete(TOKEN_KEY)?;
        *self.last.lock().unwrap_or_else(|e| e.into_inner()) = None;
        Ok(())
    }

    /// Ambil data baru. Kabar baru dibanding pengecekan sebelumnya langsung dikirim ke pulau;
    /// pengecekan pertama hanya mencatat keadaan supaya pulau tidak dibanjiri saat aplikasi dibuka.
    pub fn refresh(&self) -> Result<Radar, GithubError> {
        let Some(token) = self.secrets.get(TOKEN_KEY)? else {
            return Err(GithubError::NoToken);
        };
        let radar = self.api.radar(&token)?;
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(prev) = last.as_ref() {
            for text in news(&prev.radar, &radar).into_iter().take(MAX_NOTICES) {
                self.island.dispatch(IslandAction::Notice(text));
            }
        }
        *last = Some(Last {
            at: Instant::now(),
            radar: radar.clone(),
        });
        Ok(radar)
    }

    /// Hasil terakhir bila masih segar, selain itu ambil baru.
    pub fn radar(&self) -> Result<Radar, GithubError> {
        let fresh = self
            .last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .filter(|l| l.at.elapsed() < FRESH)
            .map(|l| l.radar.clone());
        match fresh {
            Some(r) => Ok(r),
            None => self.refresh(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{MemSecrets, services};
    use crate::domain::github::{Ci, Pr};
    use crate::domain::island::state::{Content, View};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Api(Mutex<Radar>, AtomicUsize);
    impl GithubApi for Api {
        fn radar(&self, _token: &str) -> Result<Radar, GithubError> {
            self.1.fetch_add(1, Ordering::SeqCst);
            Ok(self.0.lock().unwrap().clone())
        }
    }

    fn pr(url: &str, ci: Ci) -> Pr {
        Pr {
            repo: "a/b".into(),
            title: "t".into(),
            url: url.into(),
            ci,
        }
    }

    fn make() -> (
        Arc<GithubService>,
        Arc<Api>,
        Arc<IslandService>,
        tokio::runtime::Runtime,
    ) {
        let (_settings, island, rt) = services(Default::default());
        let api = Arc::new(Api(Mutex::new(Radar::default()), AtomicUsize::new(0)));
        let svc = GithubService::new(api.clone(), Arc::new(MemSecrets::default()), island.clone());
        (svc, api, island, rt)
    }

    #[test]
    fn tanpa_token_galat_dan_token_tidak_valid_ditolak() {
        let (svc, api, _island, _rt) = make();
        assert!(matches!(svc.refresh(), Err(GithubError::NoToken)));
        for bad in ["", "  ", "a\nb", &"t".repeat(MAX_TOKEN_LEN + 1)] {
            assert!(svc.set_token(bad).is_err(), "{bad:?}");
        }
        assert_eq!(api.1.load(Ordering::SeqCst), 0);
        assert!(!svc.has_token().unwrap());
        svc.set_token(" ghp_abc ").unwrap();
        assert!(svc.has_token().unwrap());
        svc.clear_token().unwrap();
        assert!(!svc.has_token().unwrap());
    }

    #[test]
    fn hasil_segar_dipakai_ulang_dan_kabar_hanya_setelah_pengecekan_pertama() {
        let (svc, api, island, _rt) = make();
        svc.set_token("ghp_abc").unwrap();
        api.0.lock().unwrap().review_requests = vec![pr("u1", Ci::Unknown)];
        svc.refresh().unwrap();
        assert_eq!(
            island.snapshot().view,
            View::Idle,
            "pengecekan pertama tidak memunculkan kabar"
        );
        svc.radar().unwrap();
        assert_eq!(api.1.load(Ordering::SeqCst), 1, "kedua dari cache");

        api.0.lock().unwrap().review_requests = vec![pr("u1", Ci::Unknown), pr("u2", Ci::Unknown)];
        svc.refresh().unwrap();
        assert!(matches!(
            island.snapshot().view,
            View::Expanded(Content::Notice(t)) if t.contains("Review diminta")
        ));
    }
}
