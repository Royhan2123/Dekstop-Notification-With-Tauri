use crate::domain::ports::IdleDetector;

/// Input terakhir dari OS. Platform tanpa dukungan melapor `None` (dianggap selalu aktif).
pub struct SystemIdle;

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
        #[cfg(not(windows))]
        None
    }
}
