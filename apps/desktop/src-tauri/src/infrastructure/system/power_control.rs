use crate::domain::error::PowerError;
use crate::domain::ports::PowerControl;
use crate::domain::power::PowerAction;

/// Perintah daya lewat Windows. Platform lain melapor `Unsupported`.
pub struct SystemPowerControl;

#[cfg(windows)]
fn run(program: &str, args: &[&str]) -> Result<(), PowerError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = std::process::Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| PowerError::Failed(e.to_string()))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| PowerError::Failed(format!("{program} keluar dengan {status}")))
}

/// Menunggu sampai tidak ada input selama 0,7 dtk (maks ~4 dtk, lalu lanjut apa adanya).
#[cfg(windows)]
fn wait_until_idle() {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    for _ in 0..40 {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        // SAFETY: info valid selama panggilan dan cbSize sudah diisi.
        let ok = unsafe { GetLastInputInfo(&mut info) }.as_bool();
        // SAFETY: tanpa argumen.
        let idle_ms = unsafe { GetTickCount() }.wrapping_sub(info.dwTime);
        if !ok || idle_ms >= 700 {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

impl PowerControl for SystemPowerControl {
    fn perform(&self, action: PowerAction) -> Result<(), PowerError> {
        // Hanya untuk pengujian otomatis: ISLA_POWER_DRY_RUN=1 mencatat perintah tanpa menjalankannya.
        if std::env::var_os("ISLA_POWER_DRY_RUN").is_some() {
            tracing::warn!("dry-run: perintah daya {action:?} dilewati");
            return Ok(());
        }
        #[cfg(windows)]
        {
            match action {
                PowerAction::Shutdown => run("shutdown", &["/s", "/t", "0"]),
                PowerAction::Restart => run("shutdown", &["/r", "/t", "0"]),
                PowerAction::Lock => run("rundll32.exe", &["user32.dll,LockWorkStation"]),
                PowerAction::Sleep => {
                    // Tidur = layar mati, menyala lagi oleh klik atau tombol apa saja. Bukan sleep S0/hibernasi:
                    // di laptop Modern Standby, SetSuspendState bangun hanya lewat tombol power.
                    // Tunggu tangan lepas dari mouse/keyboard (idle 0,7 dtk, maks ~4 dtk): getaran kecil sesudah klik
                    // tombol akan langsung menyalakan layar lagi dan terlihat berkedip.
                    std::thread::spawn(|| {
                        use windows::Win32::Foundation::{LPARAM, WPARAM};
                        use windows::Win32::UI::WindowsAndMessaging::{
                            HWND_BROADCAST, PostMessageW, SC_MONITORPOWER, WM_SYSCOMMAND,
                        };
                        wait_until_idle();
                        // SAFETY: tanpa pointer; lparam 2 = matikan monitor.
                        let sent = unsafe {
                            PostMessageW(
                                Some(HWND_BROADCAST),
                                WM_SYSCOMMAND,
                                WPARAM(SC_MONITORPOWER as usize),
                                LPARAM(2),
                            )
                        };
                        if let Err(e) = sent {
                            tracing::warn!("gagal mematikan layar: {e}");
                        }
                    });
                    Ok(())
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = action;
            Err(PowerError::Unsupported)
        }
    }
}
