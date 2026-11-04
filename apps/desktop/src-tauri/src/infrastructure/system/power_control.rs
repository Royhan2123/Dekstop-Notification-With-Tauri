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

impl PowerControl for SystemPowerControl {
    fn perform(&self, action: PowerAction) -> Result<(), PowerError> {
        // Build debug (dev, tes) hanya mencatat: jangan sampai pengujian benar-benar mematikan mesin.
        if cfg!(debug_assertions) && std::env::var_os("ISLA_POWER_REAL").is_none() {
            tracing::warn!(
                "mode dev: perintah daya {action:?} dilewati (set ISLA_POWER_REAL=1 untuk menjalankan)"
            );
            return Ok(());
        }
        #[cfg(windows)]
        {
            match action {
                PowerAction::Shutdown => run("shutdown", &["/s", "/t", "0"]),
                PowerAction::Restart => run("shutdown", &["/r", "/t", "0"]),
                PowerAction::Lock => run("rundll32.exe", &["user32.dll,LockWorkStation"]),
                PowerAction::Sleep => {
                    use windows::Win32::System::Power::SetSuspendState;
                    // SAFETY: panggilan tanpa pointer; false,false,false = tidur biasa (bukan hibernasi).
                    let ok = unsafe { SetSuspendState(false, false, false) };
                    if ok {
                        Ok(())
                    } else {
                        Err(PowerError::Failed("SetSuspendState gagal".into()))
                    }
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
