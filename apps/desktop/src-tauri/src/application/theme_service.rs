use std::sync::Arc;

use crate::domain::error::ThemeError;
use crate::domain::ports::{ThemeFetcher, WallpaperSource};
use crate::domain::theme::{is_public_https_url, is_theme_file};

/// Tema dari luar aplikasi: berkas tema lewat URL dan wallpaper Windows untuk latar pulau.
pub struct ThemeService {
    fetcher: Arc<dyn ThemeFetcher>,
    wallpaper: Arc<dyn WallpaperSource>,
}

impl ThemeService {
    pub fn new(fetcher: Arc<dyn ThemeFetcher>, wallpaper: Arc<dyn WallpaperSource>) -> Arc<Self> {
        Arc::new(Self { fetcher, wallpaper })
    }

    /// Isi berkas tema dari `url`. Nilainya belum disaring: penyaringan tetap oleh `Settings::sanitized`
    /// saat tema diterapkan, seperti impor dari berkas.
    pub fn fetch_theme(&self, url: &str) -> Result<String, ThemeError> {
        let url = url.trim();
        if !is_public_https_url(url) {
            return Err(ThemeError::Invalid(
                "gunakan tautan https ke situs biasa".into(),
            ));
        }
        let text = self.fetcher.fetch(url)?;
        if !is_theme_file(&text) {
            return Err(ThemeError::Invalid("bukan berkas tema Isla".into()));
        }
        Ok(text)
    }

    /// Wallpaper saat ini sebagai data URL JPEG kecil; `None` bila tidak terbaca.
    pub fn wallpaper(&self) -> Option<String> {
        self.wallpaper.current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Fake(Mutex<Result<String, ()>>, Mutex<u32>);
    impl ThemeFetcher for Fake {
        fn fetch(&self, _url: &str) -> Result<String, ThemeError> {
            *self.1.lock().unwrap() += 1;
            self.0
                .lock()
                .unwrap()
                .clone()
                .map_err(|_| ThemeError::Unavailable("putus".into()))
        }
    }
    struct NoWallpaper;
    impl WallpaperSource for NoWallpaper {
        fn current(&self) -> Option<String> {
            None
        }
    }

    fn make(reply: Result<String, ()>) -> (Arc<ThemeService>, Arc<Fake>) {
        let fake = Arc::new(Fake(Mutex::new(reply), Mutex::new(0)));
        (ThemeService::new(fake.clone(), Arc::new(NoWallpaper)), fake)
    }

    #[test]
    fn url_ditolak_sebelum_menghubungi_jaringan() {
        let (svc, fake) = make(Ok(r#"{"isla_theme":1}"#.into()));
        assert!(svc.fetch_theme("http://example.com/a.json").is_err());
        assert!(svc.fetch_theme("https://127.0.0.1/a.json").is_err());
        assert_eq!(*fake.1.lock().unwrap(), 0);
    }

    #[test]
    fn hanya_berkas_tema_yang_diteruskan() {
        let (svc, _) = make(Ok(r#"{"isla_theme":1,"theme":"ocean"}"#.into()));
        assert!(
            svc.fetch_theme(" https://example.com/t.json ")
                .unwrap()
                .contains("ocean")
        );
        let (svc, _) = make(Ok("<html>bukan tema</html>".into()));
        assert!(matches!(
            svc.fetch_theme("https://example.com/t.json"),
            Err(ThemeError::Invalid(_))
        ));
        let (svc, _) = make(Err(()));
        assert!(matches!(
            svc.fetch_theme("https://example.com/t.json"),
            Err(ThemeError::Unavailable(_))
        ));
    }
}
