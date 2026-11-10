// Hanya Windows yang punya registri aplikasi; di OS lain pembantunya tidak terpakai.
#![cfg_attr(not(windows), allow(dead_code))]
use crate::domain::ports::InstalledApps;
use crate::domain::startup::InstalledApp;

/// Aplikasi terpasang dari registri Windows: daftar "Program dan Fitur" (yang punya ikon .exe) ditambah "App Paths".
/// Aplikasi Microsoft Store (tanpa .exe biasa) tidak muncul.
pub struct RegistryApps;

/// Ganti `%NAMA%` dengan variabel lingkungan; yang tidak dikenal dibiarkan.
fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('%') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let Some(j) = after.find('%') else {
            out.push('%');
            rest = after;
            continue;
        };
        match std::env::var(&after[..j]) {
            Ok(v) => out.push_str(&v),
            Err(_) => out.push_str(&rest[i..=i + 1 + j]),
        }
        rest = &after[j + 1..];
    }
    out.push_str(rest);
    out
}

/// Path .exe dari nilai `DisplayIcon` ("C:\\a b\\x.exe",0 atau C:\\x.exe,0 atau %ProgramFiles%\\x.exe).
pub(crate) fn exe_from_icon(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let path = match raw.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?,
        None => match raw.rsplit_once(',') {
            Some((p, idx))
                if idx
                    .trim()
                    .trim_start_matches('-')
                    .chars()
                    .all(|c| c.is_ascii_digit()) =>
            {
                p
            }
            _ => raw,
        },
    };
    let path = expand_env(path.trim());
    path.to_ascii_lowercase().ends_with(".exe").then_some(path)
}

/// Nama berkas yang jelas bukan aplikasi untuk dibuka sendiri: pemasang, pencopot, pembaruan, pembantu, layanan.
const NOISE: [&str; 6] = ["unins", "setup", "install", "update", "helper", "service"];

/// Buang yang tidak ada di disk, bukan aplikasi (lihat `NOISE`) atau berasal dari cache pemasang, hilangkan
/// duplikat (path sama; yang lebih awal menang), lalu urutkan menurut nama.
pub(crate) fn tidy(apps: Vec<InstalledApp>, exists: impl Fn(&str) -> bool) -> Vec<InstalledApp> {
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<InstalledApp> = apps
        .into_iter()
        .filter(|a| {
            let p = a.path.to_ascii_lowercase();
            let file = p.rsplit(['\\', '/']).next().unwrap_or(&p);
            !a.name.trim().is_empty()
                && !NOISE.iter().any(|n| file.contains(n))
                && !p.contains("\\windows\\installer\\")
                && !p.contains("\\package cache\\")
                && exists(&a.path)
                && seen.insert(p)
        })
        .collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

#[cfg(windows)]
fn collect() -> Vec<InstalledApp> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const UNINSTALL: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    const UNINSTALL32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";
    const APP_PATHS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";

    let mut apps = Vec::new();
    for (hive, sub) in [
        (HKEY_LOCAL_MACHINE, UNINSTALL),
        (HKEY_LOCAL_MACHINE, UNINSTALL32),
        (HKEY_CURRENT_USER, UNINSTALL),
    ] {
        let Ok(key) = RegKey::predef(hive).open_subkey(sub) else {
            continue;
        };
        for name in key.enum_keys().flatten() {
            let Ok(app) = key.open_subkey(&name) else {
                continue;
            };
            // Komponen sistem dan tambalan tidak muncul di Program dan Fitur.
            if app.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1
                || app.get_value::<String, _>("ParentKeyName").is_ok()
            {
                continue;
            }
            let (Ok(display), Ok(icon)) = (
                app.get_value::<String, _>("DisplayName"),
                app.get_value::<String, _>("DisplayIcon"),
            ) else {
                continue;
            };
            if let Some(path) = exe_from_icon(&icon) {
                apps.push(InstalledApp {
                    name: display.trim().to_string(),
                    path,
                });
            }
        }
    }
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        let Ok(key) = RegKey::predef(hive).open_subkey(APP_PATHS) else {
            continue;
        };
        for exe in key.enum_keys().flatten() {
            let Ok(app) = key.open_subkey(&exe) else {
                continue;
            };
            let Ok(raw) = app.get_value::<String, _>("") else {
                continue;
            };
            if let Some(path) = exe_from_icon(&raw) {
                apps.push(InstalledApp {
                    name: crate::domain::startup::name_from_path(&exe),
                    path,
                });
            }
        }
    }
    apps
}

impl InstalledApps for RegistryApps {
    fn list(&self) -> Vec<InstalledApp> {
        #[cfg(windows)]
        {
            tidy(collect(), |p| std::path::Path::new(p).is_file())
        }
        #[cfg(not(windows))]
        {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_exe_diambil_dari_nilai_ikon() {
        assert_eq!(
            exe_from_icon("\"C:\\A B\\x.exe\",0").as_deref(),
            Some("C:\\A B\\x.exe")
        );
        assert_eq!(
            exe_from_icon("C:\\Apps\\y.EXE,-12").as_deref(),
            Some("C:\\Apps\\y.EXE")
        );
        assert_eq!(
            exe_from_icon("C:\\Apps\\z.exe").as_deref(),
            Some("C:\\Apps\\z.exe")
        );
        assert_eq!(exe_from_icon("C:\\Apps\\ikon.ico"), None);
        assert_eq!(exe_from_icon(""), None);
        assert_eq!(
            exe_from_icon("%TIDAK_ADA_VAR%\\q.exe").as_deref(),
            Some("%TIDAK_ADA_VAR%\\q.exe")
        );
    }

    #[test]
    fn variabel_lingkungan_diperluas() {
        // SAFETY: tes ini satu-satunya yang memakai variabel ini.
        unsafe { std::env::set_var("ISLA_TES_DIR", "D:\\Prog") };
        assert_eq!(
            exe_from_icon("%ISLA_TES_DIR%\\app.exe,0").as_deref(),
            Some("D:\\Prog\\app.exe")
        );
        assert_eq!(expand_env("100%"), "100%");
    }

    fn app(name: &str, path: &str) -> InstalledApp {
        InstalledApp {
            name: name.into(),
            path: path.into(),
        }
    }

    #[test]
    fn daftar_dirapikan_tanpa_duplikat_pencopot_dan_yang_hilang() {
        let apps = vec![
            app("Zoom", "C:\\z\\zoom.exe"),
            app("Google Chrome", "C:\\g\\chrome.exe"),
            app("chrome", "C:\\G\\Chrome.exe"),
            app("Hapus Zoom", "C:\\z\\uninstall.exe"),
            app("Pencopot", "C:\\q\\unins000.exe"),
            app("Pemasang", "C:\\q\\AppSetup.exe"),
            app(".NET", "C:\\ProgramData\\Package Cache\\{1}\\dotnet.exe"),
            app("Layanan", "C:\\q\\agent-service.exe"),
            app("Hilang", "C:\\h\\none.exe"),
            app("", "C:\\k\\kosong.exe"),
            app("Installer", "C:\\Windows\\Installer\\{1}\\i.exe"),
        ];
        let out = tidy(apps, |p| !p.contains("none"));
        let names: Vec<_> = out.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Google Chrome", "Zoom"]);
    }
}
