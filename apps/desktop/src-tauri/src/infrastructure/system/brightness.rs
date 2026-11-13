use crate::domain::ports::BrightnessReader;

/// Kecerahan layar internal. macOS belum didukung (selalu `None`).
pub struct SystemBrightness;

#[cfg(windows)]
mod wmi {
    use std::cell::RefCell;

    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        CoSetProxyBlanket, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
    };
    use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::System::Wmi::{
        IWbemLocator, IWbemServices, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
        WBEM_INFINITE, WbemLocator,
    };
    use windows::core::{BSTR, w};

    thread_local! {
        /// Sambungan WMI per thread (COM diinisialisasi per thread), dipakai ulang antar pembacaan.
        static SERVICES: RefCell<Option<IWbemServices>> = const { RefCell::new(None) };
    }

    fn connect() -> Option<IWbemServices> {
        // SAFETY: panggilan COM standar; hasil `CoInitializeEx` boleh diabaikan (S_FALSE bila sudah
        // diinisialisasi di thread ini), dan semua objek dilepas otomatis oleh pembungkus `windows`.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let locator: IWbemLocator =
                CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).ok()?;
            let svc = locator
                .ConnectServer(
                    &BSTR::from("ROOT\\WMI"),
                    &BSTR::new(),
                    &BSTR::new(),
                    &BSTR::new(),
                    0,
                    &BSTR::new(),
                    None,
                )
                .ok()?;
            CoSetProxyBlanket(
                &svc,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHZ_NONE,
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )
            .ok()?;
            Some(svc)
        }
    }

    fn query(svc: &IWbemServices) -> Option<u8> {
        // SAFETY: enumerator dan objek hidup selama blok ini; `Next` mengisi satu elemen `objs`.
        unsafe {
            let rows = svc
                .ExecQuery(
                    &BSTR::from("WQL"),
                    &BSTR::from(
                        "SELECT CurrentBrightness FROM WmiMonitorBrightness WHERE Active = TRUE",
                    ),
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                    None,
                )
                .ok()?;
            let mut objs = [None];
            let mut returned = 0;
            rows.Next(WBEM_INFINITE, &mut objs, &mut returned)
                .ok()
                .ok()?;
            let obj = objs[0].take()?;
            let mut value = VARIANT::default();
            obj.Get(w!("CurrentBrightness"), 0, &mut value, None, None)
                .ok()?;
            u8::try_from(u32::try_from(&value).ok()?).ok()
        }
    }

    /// Satu pembacaan; sambungan dibuat ulang sekali bila yang lama sudah tidak berlaku.
    pub fn read() -> Option<u8> {
        SERVICES.with(|cell| {
            let mut slot = cell.borrow_mut();
            for _ in 0..2 {
                if slot.is_none() {
                    *slot = connect();
                }
                if let Some(v) = slot.as_ref().and_then(query) {
                    return Some(v);
                }
                *slot = None;
            }
            None
        })
    }
}

/// Kecerahan layar internal macOS dari `ioreg -rc AppleBacklightDisplay`, baris
/// `"brightness"={"max"=1024,"min"=0,"value"=858}`. Kunci lain yang mirip (`linear-brightness`,
/// `brightness-probe`) sengaja dilewati: yang dicari diawali tanda kutip. Mac Apple Silicon dan layar
/// eksternal tidak punya node ini, jadi hasilnya `None`.
#[cfg(any(target_os = "macos", test))]
fn parse_ioreg_brightness(out: &str) -> Option<u8> {
    let start = out.find("\"brightness\"={")? + "\"brightness\"={".len();
    let body = &out[start..out[start..].find('}')? + start];
    let field = |name: &str| -> Option<f64> {
        let key = format!("\"{name}\"=");
        // `"max"=` bisa muncul di awal atau tengah; ambil angka setelah tanda sama dengan.
        let i = body.find(&key)? + key.len();
        let num: String = body[i..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        num.parse().ok()
    };
    let (value, max) = (field("value")?, field("max")?);
    (max > 0.0).then(|| (value / max * 100.0).round().clamp(0.0, 100.0) as u8)
}

impl BrightnessReader for SystemBrightness {
    fn read(&self) -> Option<u8> {
        #[cfg(windows)]
        {
            wmi::read().filter(|v| *v <= 100)
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
        #[cfg(target_os = "macos")]
        {
            let out = std::process::Command::new("ioreg")
                .args(["-rc", "AppleBacklightDisplay"])
                .output()
                .ok()?;
            parse_ioreg_brightness(&String::from_utf8_lossy(&out.stdout))
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioreg_kecerahan_diurai_dan_kunci_mirip_dilewati() {
        let out = r#""IODisplayParameters" = {"linear-brightness"={"min"=0,"max"=906,"value"=386},"brightness-probe"={"min"=0,"max"=1024,"value"=111},"brightness"={"max"=1024,"min"=0,"value"=858},"brightness-fade"={"min"=0,"max"=1023,"value"=0}}"#;
        assert_eq!(parse_ioreg_brightness(out), Some(84)); // 858 / 1024
        let full = r#""brightness"={"max"=100,"min"=0,"value"=100}"#;
        assert_eq!(parse_ioreg_brightness(full), Some(100));
        assert_eq!(parse_ioreg_brightness(""), None);
        // hanya kunci mirip, tanpa "brightness" polos
        assert_eq!(
            parse_ioreg_brightness(r#""linear-brightness"={"max"=906,"min"=0,"value"=386}"#),
            None
        );
        assert_eq!(
            parse_ioreg_brightness(r#""brightness"={"max"=0,"min"=0,"value"=5}"#),
            None
        );
    }

    #[test]
    fn membaca_kecerahan_tidak_crash_dan_dalam_rentang() {
        // PC desktop atau layar eksternal tidak punya nilai ini; yang penting tidak panik.
        if let Some(v) = SystemBrightness.read() {
            assert!(v <= 100);
        }
    }
}
