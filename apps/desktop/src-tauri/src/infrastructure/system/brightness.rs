use crate::domain::ports::BrightnessReader;

/// Kecerahan layar internal. macOS belum didukung (selalu `None`).
pub struct SystemBrightness;

/// Ambil angka 0-100 dari keluaran perintah; selain itu dianggap tidak tersedia.
fn parse_percent(text: &str) -> Option<u8> {
    text.trim()
        .lines()
        .next()?
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|v| *v <= 100)
}

impl BrightnessReader for SystemBrightness {
    fn read(&self) -> Option<u8> {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // shortcut: PowerShell+WMI tiap menit (~0,3 dtk); ganti COM langsung bila terasa berat.
            let out = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction SilentlyContinue).CurrentBrightness",
                ])
                .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
                .output()
                .ok()?;
            parse_percent(&String::from_utf8_lossy(&out.stdout))
        }
        #[cfg(target_os = "linux")]
        {
            let dir = std::fs::read_dir("/sys/class/backlight")
                .ok()?
                .flatten()
                .next()?
                .path();
            let read = |name: &str| {
                std::fs::read_to_string(dir.join(name))
                    .ok()?
                    .trim()
                    .parse::<u32>()
                    .ok()
            };
            let (now, max) = (read("brightness")?, read("max_brightness")?);
            (max > 0).then(|| (now * 100 / max).min(100) as u8)
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keluaran_diurai_atau_dianggap_tidak_tersedia() {
        assert_eq!(parse_percent("70\r\n"), Some(70));
        assert_eq!(parse_percent(""), None);
        assert_eq!(parse_percent("kosong"), None);
        assert_eq!(parse_percent("250"), None);
    }
}
