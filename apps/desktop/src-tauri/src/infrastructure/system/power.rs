use crate::domain::ports::{PowerReader, PowerStatus};

/// Status baterai dan charger dari OS. Platform tanpa dukungan, atau PC tanpa baterai, melapor `None`.
pub struct SystemPower;

impl PowerReader for SystemPower {
    fn status(&self) -> Option<PowerStatus> {
        #[cfg(windows)]
        {
            use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
            let mut s = SYSTEM_POWER_STATUS::default();
            // SAFETY: `s` hidup selama pemanggilan dan API hanya menulis ke struct tersebut.
            unsafe { GetSystemPowerStatus(&mut s) }.ok()?;
            // BatteryFlag bit 128 = tidak ada baterai; ACLineStatus 255 = tidak diketahui.
            if s.BatteryFlag & 128 != 0 || s.ACLineStatus == 255 {
                return None;
            }
            Some(PowerStatus {
                plugged: s.ACLineStatus == 1,
                percent: (s.BatteryLifePercent <= 100).then_some(s.BatteryLifePercent),
            })
        }
        #[cfg(not(windows))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membaca_status_daya_tidak_crash_dan_persen_dalam_rentang() {
        if let Some(s) = SystemPower.status() {
            assert!(s.percent.is_none_or(|p| p <= 100));
        }
    }
}
