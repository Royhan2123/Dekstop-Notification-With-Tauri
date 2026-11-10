use crate::domain::ports::IdleDetector;

/// Input terakhir dari OS. Platform tanpa dukungan melapor `None` (dianggap selalu aktif).
pub struct SystemIdle;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
}
impl IdleDetector for SystemIdle {
    fn idle_seconds(&self) -> Option<u64> {
        #[cfg(windows)]
        {
            use windows::Win32::System::SystemInformation::GetTickCount;
            use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
            let mut info = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            // SAFETY: `info` hidup selama pemanggilan dan `cbSize` sudah diisi sesuai syarat API.
            let ok = unsafe { GetLastInputInfo(&mut info) }.as_bool();
            // Selisih tick 32-bit boleh melilit (setelah ~49 hari), jadi pakai wrapping_sub.
            let idle_ms = unsafe { GetTickCount() }.wrapping_sub(info.dwTime);
            ok.then_some(u64::from(idle_ms) / 1000)
        }
        #[cfg(target_os = "macos")]
        {
            // kCGEventSourceStateCombinedSessionState (0), kCGAnyInputEventType (~0): detik sejak
            // input apa pun (tombol, mouse, trackpad) di sesi pengguna.
            // SAFETY: fungsi murni CoreGraphics tanpa penunjuk; hanya mengembalikan angka.
            let secs = unsafe { CGEventSourceSecondsSinceLastEventType(0, u32::MAX) };
            (secs.is_finite() && secs >= 0.0).then_some(secs as u64)
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        None
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests_macos {
    use super::*;

    #[test]
    fn idle_macos_terbaca_dan_wajar() {
        // Mesin tanpa sesi grafis (CI) bisa melapor sangat besar atau None; tidak boleh panik.
        if let Some(s) = SystemIdle.idle_seconds() {
            assert!(s < 10 * 365 * 24 * 3600);
        }
    }
}
