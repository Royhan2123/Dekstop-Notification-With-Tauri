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

#[cfg(not(windows))]
fn wallpaper_path() -> Option<std::path::PathBuf> {
    // shortcut: wallpaper hanya dibaca di Windows; Linux/macOS menyusul bersama adapter lainnya.
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
    fn membaca_wallpaper_tidak_crash() {
        let _ = SystemWallpaper.current();
    }
}
