use crate::domain::ports::WallpaperSource;
use crate::infrastructure::media::artwork;

/// Sisi terpanjang wallpaper yang dikirim ke pulau; cukup untuk kartu 440 px pada layar rapat.
const MAX_SIDE: u32 = 640;

/// Wallpaper desktop Windows, dibaca dari berkas yang sedang dipakai dan dikecilkan.
pub struct SystemWallpaper;

#[cfg(windows)]
fn wallpaper_path() -> Option<std::path::PathBuf> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    };
    let mut buf = [0u16; 520];
    // SAFETY: `buf` hidup selama panggilan dan ukurannya (dalam karakter) sesuai uiParam.
    unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buf.len() as u32,
            Some(buf.as_mut_ptr().cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()?;
    let len = buf.iter().position(|&c| c == 0)?;
    (len > 0).then(|| String::from_utf16_lossy(&buf[..len]).into())
}

/// Keluaran `osascript` -> path wallpaper. Pada macOS terbaru System Events membalas `missing value`
/// untuk wallpaper dinamis/Aerial; itu dan apa pun yang bukan path absolut ditolak.
#[cfg(any(target_os = "macos", test))]
fn parse_wallpaper_output(out: &str) -> Option<std::path::PathBuf> {
    let path = out.trim();
    (path.starts_with('/') && !path.contains('\n')).then(|| path.into())
}

/// macOS: path wallpaper desktop utama lewat System Events (butuh izin Automation sekali).
/// Hanya JPEG/PNG yang terbaca pengolah gambar; wallpaper dinamis (HEIC) atau yang tidak diekspos
/// System Events menghasilkan `None`.
#[cfg(target_os = "macos")]
fn wallpaper_path() -> Option<std::path::PathBuf> {
    let out = std::process::Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to get picture of desktop 1"#,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_wallpaper_output(&String::from_utf8_lossy(&out.stdout))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn wallpaper_path() -> Option<std::path::PathBuf> {
    // shortcut: Linux belum didukung.
    None
}

impl WallpaperSource for SystemWallpaper {
    fn current(&self) -> Option<String> {
        let bytes = std::fs::read(wallpaper_path()?).ok()?;
        artwork::data_url(&bytes, MAX_SIDE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keluaran_wallpaper_macos_hanya_path_absolut_yang_diterima() {
        assert_eq!(
            parse_wallpaper_output("/Users/mac/Pictures/lautan.jpg\n"),
            Some("/Users/mac/Pictures/lautan.jpg".into())
        );
        for bad in ["missing value", "", "   ", "lautan.jpg", "/a.jpg\n/b.jpg"] {
            assert_eq!(parse_wallpaper_output(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn membaca_wallpaper_tidak_crash() {
        let _ = SystemWallpaper.current();
    }
}
