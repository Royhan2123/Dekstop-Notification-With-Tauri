use crate::domain::ports::{PowerReader, PowerStatus};

/// Status baterai dan charger dari OS. Platform tanpa dukungan, atau PC tanpa baterai, melapor `None`.
pub struct SystemPower;

/// Mengurai keluaran `pmset -g batt`:
/// ```text
/// Now drawing from 'AC Power'
///  -InternalBattery-0 (id=7077987)    100%; charged; 0:00 remaining present: true
/// ```
/// Mac tanpa baterai (iMac, Mac mini) tidak punya baris baterai, jadi hasilnya `None` seperti PC desktop.
#[cfg(any(target_os = "macos", test))]
fn parse_pmset(out: &str) -> Option<PowerStatus> {
    let first = out.lines().next()?;
    let plugged = if first.contains("'AC Power'") {
        true
    } else if first.contains("'Battery Power'") {
        false
    } else {
        return None;
    };
    let battery = out
        .lines()
        .find(|l| l.contains("InternalBattery") && l.contains('%'))?;
    let before = battery.split('%').next()?;
    let digits: String = before
        .chars()
        .rev()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let percent = digits.parse::<u8>().ok().filter(|p| *p <= 100);
    Some(PowerStatus { plugged, percent })
}
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
        #[cfg(target_os = "macos")]
        {
            let out = std::process::Command::new("pmset")
                .args(["-g", "batt"])
                .output()
                .ok()?;
            parse_pmset(&String::from_utf8_lossy(&out.stdout))
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pmset_charger_dan_baterai_diurai() {
        let ac = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=7077987)\t100%; charged; 0:00 remaining present: true\n";
        assert_eq!(
            parse_pmset(ac),
            Some(PowerStatus {
                plugged: true,
                percent: Some(100)
            })
        );
        let batt = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=1)\t7%; discharging; 0:42 remaining present: true\n";
        assert_eq!(
            parse_pmset(batt),
            Some(PowerStatus {
                plugged: false,
                percent: Some(7)
            })
        );
    }

    #[test]
    fn pmset_tanpa_baterai_atau_rusak_menjadi_none() {
        // Mac desktop: hanya baris sumber daya.
        assert_eq!(parse_pmset("Now drawing from 'AC Power'\n"), None);
        assert_eq!(parse_pmset(""), None);
        assert_eq!(
            parse_pmset("sesuatu yang lain\n -InternalBattery-0 50%; x"),
            None
        );
    }

    #[test]
    fn membaca_status_daya_tidak_crash_dan_persen_dalam_rentang() {
        if let Some(s) = SystemPower.status() {
            assert!(s.percent.is_none_or(|p| p <= 100));
        }
    }
}
