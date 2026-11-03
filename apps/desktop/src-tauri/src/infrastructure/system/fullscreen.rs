use crate::domain::ports::FullscreenDetector;

/// Deteksi lewat status notifikasi pengguna milik Shell Windows: "sibuk" (aplikasi layar penuh),
/// game Direct3D layar penuh, atau mode presentasi. Platform lain melapor tidak layar penuh.
pub struct SystemFullscreen;

impl FullscreenDetector for SystemFullscreen {
    fn is_fullscreen(&self) -> bool {
        #[cfg(windows)]
        {
            use windows::Win32::UI::Shell::{
                QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
                SHQueryUserNotificationState,
            };
            // SAFETY: panggilan tanpa argumen penunjuk; hanya mengembalikan nilai enum.
            unsafe { SHQueryUserNotificationState() }.is_ok_and(|s| {
                matches!(
                    s,
                    QUNS_BUSY | QUNS_PRESENTATION_MODE | QUNS_RUNNING_D3D_FULL_SCREEN
                )
            })
        }
        #[cfg(not(windows))]
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membaca_status_layar_penuh_tidak_crash() {
        let _ = SystemFullscreen.is_fullscreen();
    }
}
