//! Menjalankan Jarvis (asisten suara, program terpisah) dari dalam Isla.
//!
//! Jarvis dipasang oleh skrip pemasangnya ke `<data dir>/jarvis-app/` beserta peluncur
//! `run.sh` / `run.cmd`. Isla hanya mencari peluncur itu, menjalankannya di latar, dan mematikannya
//! saat Isla keluar. Jarvis opsional: tanpa pemasangan, tidak ada yang terjadi.
use std::fs::OpenOptions;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use serde::Deserialize;

use super::jarvis::api_file;

/// Satu-satunya proses Jarvis yang dijalankan Isla (bila ada).
static CHILD: Mutex<Option<Child>> = Mutex::new(None);

#[derive(Debug, PartialEq, Eq)]
pub enum Started {
    /// Jarvis sudah menyala (dijalankan pihak lain), tidak disentuh.
    AlreadyRunning,
    /// Isla menjalankannya sekarang.
    Launched,
}

/// Folder pemasangan Jarvis: bersebelahan dengan folder data `jarvis` milik Jarvis sendiri.
pub fn home_dir() -> Option<PathBuf> {
    Some(api_file()?.parent()?.parent()?.join("jarvis-app"))
}

/// Peluncur hasil pemasangan, `None` bila Jarvis belum terpasang.
pub fn launcher() -> Option<PathBuf> {
    let name = if cfg!(windows) { "run.cmd" } else { "run.sh" };
    home_dir().map(|d| d.join(name)).filter(|p| p.is_file())
}

#[derive(Deserialize)]
struct Port {
    port: u16,
}

/// Jarvis menjawab di port yang tertulis di `api.json`? (berkas basi tidak dihitung)
pub fn is_running() -> bool {
    let Some(bytes) = api_file().and_then(|p| std::fs::read(p).ok()) else {
        return false;
    };
    let Ok(Port { port }) = serde_json::from_slice(&bytes) else {
        return false;
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

/// Jalankan Jarvis di latar bila terpasang dan belum menyala. Keluarannya ke `jarvis.log`.
pub fn start() -> Result<Started, String> {
    if is_running() {
        return Ok(Started::AlreadyRunning);
    }
    let launcher = launcher().ok_or_else(|| "Jarvis belum terpasang".to_string())?;
    let mut slot = CHILD.lock().map_err(|e| e.to_string())?;
    if let Some(child) = slot.as_mut()
        && matches!(child.try_wait(), Ok(None))
    {
        return Ok(Started::Launched); // sudah dijalankan Isla, masih bersiap
    }
    let log = home_dir()
        .map(|d| d.join("jarvis.log"))
        .and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
    let (out, err) = match log.and_then(|f| f.try_clone().ok().map(|c| (f, c))) {
        Some((a, b)) => (Stdio::from(a), Stdio::from(b)),
        None => (Stdio::null(), Stdio::null()),
    };
    let mut cmd = Command::new(&launcher);
    cmd.stdin(Stdio::null()).stdout(out).stderr(err);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let child = cmd
        .spawn()
        .map_err(|e| format!("Gagal menjalankan Jarvis: {e}"))?;
    *slot = Some(child);
    Ok(Started::Launched)
}

/// Matikan Jarvis yang dijalankan Isla (yang dijalankan pihak lain dibiarkan).
pub fn stop() {
    let Ok(mut slot) = CHILD.lock() else { return };
    let Some(mut child) = slot.take() else { return };
    let pid = child.id().to_string();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .status();
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill").args(["-TERM", &pid]).status();
        for _ in 0..30 {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_pemasangan_bersebelahan_dengan_data_jarvis() {
        let h = home_dir().expect("folder data tersedia");
        assert!(h.ends_with("jarvis-app"));
        assert_eq!(h.parent(), api_file().unwrap().parent().unwrap().parent());
    }

    #[test]
    fn tanpa_peluncur_start_gagal_dengan_pesan_jelas() {
        if launcher().is_some() || is_running() {
            return; // mesin pengembang yang sudah memasang Jarvis
        }
        assert_eq!(start().unwrap_err(), "Jarvis belum terpasang");
    }

    #[test]
    fn stop_tanpa_proses_tidak_apa_apa() {
        stop();
        stop();
    }
}
