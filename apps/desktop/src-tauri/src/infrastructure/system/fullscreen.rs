use crate::domain::ports::FullscreenDetector;

/// Deteksi lewat status notifikasi pengguna milik Shell Windows: "sibuk" (aplikasi layar penuh),
/// game Direct3D layar penuh, atau mode presentasi. Platform lain melapor tidak layar penuh.
pub struct SystemFullscreen;

/// Jendela biasa (lapisan 0) yang persis menutupi seluruh layar utama dari pojok kiri atas = aplikasi
/// layar penuh (video, game, presentasi). Jendela yang dimaksimalkan biasa menyisakan menu bar,
/// jadi tidak lolos. Lapisan negatif (wallpaper, ikon desktop) dan lapisan atas (menu bar, pulau)
/// diabaikan.
#[cfg(any(target_os = "macos", test))]
fn covers_display(layer: i64, x: f64, y: f64, w: f64, h: f64, display: (f64, f64)) -> bool {
    layer == 0
        && x.abs() < 1.0
        && y.abs() < 1.0
        && (w - display.0).abs() < 1.0
        && (h - display.1).abs() < 1.0
}

#[cfg(target_os = "macos")]
fn macos_fullscreen() -> bool {
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::string::CFString;
    use core_graphics::display::CGDisplay;
    use core_graphics::window::{
        copy_window_info, kCGNullWindowID, kCGWindowListExcludeDesktopElements,
        kCGWindowListOptionOnScreenOnly,
    };

    let Some(list) = copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    ) else {
        return false;
    };
    let screen = CGDisplay::main().bounds();
    let display = (screen.size.width, screen.size.height);
    let (layer_key, bounds_key) = (
        CFString::from_static_string("kCGWindowLayer"),
        CFString::from_static_string("kCGWindowBounds"),
    );
    for item in list.iter() {
        // SAFETY: tiap elemen CGWindowListCopyWindowInfo adalah CFDictionary yang hidup selama `list`.
        let window: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_get_rule(*item as CFDictionaryRef) };
        let layer = window
            .find(&layer_key)
            .and_then(|v| v.downcast::<CFNumber>())
            .and_then(|n| n.to_i64())
            .unwrap_or(-1);
        if layer != 0 {
            continue;
        }
        let Some(bounds) = window
            .find(&bounds_key)
            .and_then(|v| v.downcast::<CFDictionary>())
        else {
            continue;
        };
        let get = |name: &'static str| -> Option<f64> {
            let key = CFString::from_static_string(name);
            let ptr = bounds.find(key.as_concrete_TypeRef() as *const std::ffi::c_void)?;
            // SAFETY: nilai kotak batas jendela adalah CFNumber.
            let n: CFNumber = unsafe { CFNumber::wrap_under_get_rule(*ptr as CFNumberRef) };
            n.to_f64()
        };
        if let (Some(x), Some(y), Some(w), Some(h)) =
            (get("X"), get("Y"), get("Width"), get("Height"))
            && covers_display(layer, x, y, w, h, display)
        {
            return true;
        }
    }
    false
}
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
        #[cfg(target_os = "macos")]
        {
            macos_fullscreen()
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanya_jendela_biasa_yang_menutupi_seluruh_layar_dihitung_layar_penuh() {
        let screen = (1792.0, 1120.0);
        assert!(covers_display(0, 0.0, 0.0, 1792.0, 1120.0, screen));
        // dimaksimalkan biasa: menyisakan menu bar
        assert!(!covers_display(0, 0.0, 25.0, 1792.0, 1095.0, screen));
        // jendela ukuran biasa
        assert!(!covers_display(0, 100.0, 100.0, 900.0, 700.0, screen));
        // wallpaper/desktop (lapisan negatif) dan pulau/menu bar (lapisan atas)
        assert!(!covers_display(
            -2147483624,
            0.0,
            0.0,
            1792.0,
            1120.0,
            screen
        ));
        assert!(!covers_display(25, 0.0, 0.0, 1792.0, 1120.0, screen));
    }

    #[test]
    fn membaca_status_layar_penuh_tidak_crash() {
        let _ = SystemFullscreen.is_fullscreen();
    }
}
