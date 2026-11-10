//! Mode hook: `isla-desk hook pre-push --ipc <berkas> <remote> <url>`, dipanggil oleh git.
//! Kode keluar 0 melanjutkan push, 1 menghentikannya. Bila aplikasi tidak berjalan, push lanjut.
use std::io::BufRead;
use std::path::PathBuf;

use super::ipc;
use crate::domain::bridge::clean_query;

pub fn run_hook(args: &[String]) -> i32 {
    attach_parent_console();
    let ipc_file = args
        .iter()
        .position(|a| a == "--ipc")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let (Some(ipc_file), Ok(repo)) = (ipc_file, std::env::current_dir()) else {
        return 0;
    };
    let refs: Vec<String> = std::io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .collect();
    if refs.is_empty() || !ipc_file.exists() {
        return 0;
    }
    eprintln!("Isla: mereview push; keputusan ada di pulau...");
    match ipc::ask(&ipc_file, &repo.to_string_lossy(), refs) {
        Some(res) if !res.allow => {
            eprintln!("Isla: {}", res.message);
            1
        }
        _ => 0,
    }
}

/// `isla-desk notify [--ipc <berkas>] <teks...>`: tampilkan pemberitahuan di pulau (mis. perintah
/// terminal selesai). Kode keluar 0 bila terkirim, 1 bila aplikasi tidak berjalan atau teks tidak sah.
pub fn run_notify(args: &[String]) -> i32 {
    attach_parent_console();
    let (ipc_file, text) = match args.first().map(String::as_str) {
        Some("--ipc") if args.len() >= 2 => (Some(PathBuf::from(&args[1])), args[2..].join(" ")),
        _ => (None, args.join(" ")),
    };
    let ipc_file = ipc_file.or_else(default_ipc_file);
    let (Some(ipc_file), Some(text)) = (ipc_file, clean_query(&text)) else {
        eprintln!("Pemakaian: isla-desk notify [--ipc <berkas>] <teks>");
        return 1;
    };
    if ipc::notify(&ipc_file, &text) {
        0
    } else {
        eprintln!("Isla: aplikasi tidak berjalan");
        1
    }
}

/// `isla-desk commit-msg [--ipc <berkas>]`: cetak saran pesan commit untuk perubahan staged di folder
/// saat ini, mis. `git commit -m "$(isla-desk commit-msg)"`. Hanya pesannya yang ke stdout.
pub fn run_commit_msg(args: &[String]) -> i32 {
    attach_parent_console();
    let ipc_file = match args.first().map(String::as_str) {
        Some("--ipc") if args.len() >= 2 => Some(PathBuf::from(&args[1])),
        _ => None,
    };
    let (Some(ipc_file), Ok(repo)) = (ipc_file.or_else(default_ipc_file), std::env::current_dir())
    else {
        eprintln!("Pemakaian: isla-desk commit-msg [--ipc <berkas>]");
        return 1;
    };
    match ipc::commit_message(&ipc_file, &repo.to_string_lossy()) {
        Some(res) if res.allow => {
            println!("{}", res.message);
            0
        }
        Some(res) => {
            eprintln!("Isla: {}", res.message);
            1
        }
        None => {
            eprintln!("Isla: aplikasi tidak berjalan");
            1
        }
    }
}

/// `isla-desk run [--ipc <berkas>] <perintah...>`: jalankan perintah seperti biasa lalu kabari pulau
/// berhasil atau gagal beserta lamanya (mis. `isla-desk run cargo build`). Kode keluar = kode perintahnya.
pub fn run_run(args: &[String]) -> i32 {
    attach_parent_console();
    let (ipc_file, command) = match args.first().map(String::as_str) {
        Some("--ipc") if args.len() >= 3 => (Some(PathBuf::from(&args[1])), &args[2..]),
        _ => (None, args),
    };
    if command.is_empty() {
        eprintln!("Pemakaian: isla-desk run [--ipc <berkas>] <perintah...>");
        return 1;
    }
    let started = std::time::Instant::now();
    // shortcut: di Windows lewat cmd /C supaya npm.cmd dan sejenisnya ketemu; argumen berspasi perlu diberi tanda kutip sendiri.
    let mut child = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.arg("/C").args(command);
        c
    } else {
        let mut c = std::process::Command::new(&command[0]);
        c.args(&command[1..]);
        c
    };
    let code = match child.status() {
        Ok(status) => status.code(),
        Err(e) => {
            eprintln!("Isla: perintah tidak bisa dijalankan: {e}");
            None
        }
    };
    let text = describe_run(command, code, started.elapsed());
    if let Some(ipc_file) = ipc_file.or_else(default_ipc_file) {
        let _ = ipc::notify(&ipc_file, &text);
    }
    code.unwrap_or(1)
}

/// Kalimat untuk pulau: `✓ cargo build selesai (2 mnt 3 dtk)` atau `✕ cargo build gagal, kode 101 (12 dtk)`.
fn describe_run(command: &[String], code: Option<i32>, elapsed: std::time::Duration) -> String {
    let name: Vec<&str> = command.iter().take(3).map(String::as_str).collect();
    let name = name.join(" ");
    let secs = elapsed.as_secs();
    let took = if secs >= 60 {
        format!("{} mnt {} dtk", secs / 60, secs % 60)
    } else {
        format!("{secs} dtk")
    };
    match code {
        Some(0) => format!("✓ {name} selesai ({took})"),
        Some(c) => format!("✕ {name} gagal, kode {c} ({took})"),
        None => format!("✕ {name} tidak berjalan"),
    }
}

/// Folder data aplikasi menurut aturan Tauri: %APPDATA% (Windows), ~/Library/Application Support
/// (macOS), $XDG_DATA_HOME atau ~/.local/share (Linux).
fn data_base() -> Option<PathBuf> {
    #[cfg(windows)]
    return std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    return std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    return std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
}

/// Berkas IPC bawaan aplikasi (sama dengan `data_dir/ipc.json` yang ditulis aplikasi).
fn default_ipc_file() -> Option<PathBuf> {
    data_base().map(|d| d.join("com.royhan.isladesk").join("ipc.json"))
}

/// Build rilis berjenis aplikasi GUI; tanpa ini `eprintln!` tidak terlihat di terminal pemanggil.
fn attach_parent_console() {
    #[cfg(windows)]
    // SAFETY: pemanggilan API Win32 tanpa pointer; gagal (tidak ada konsol induk) boleh diabaikan.
    unsafe {
        use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(test)]
mod tests {
    use super::{default_ipc_file, describe_run};
    use std::time::Duration;

    #[test]
    fn berkas_ipc_bawaan_memakai_folder_data_aplikasi() {
        let path = default_ipc_file().expect("HOME/APPDATA harus ada saat tes");
        assert!(path.ends_with("com.royhan.isladesk/ipc.json") || path.ends_with("ipc.json"));
        assert_eq!(
            path.parent().and_then(|p| p.file_name()).unwrap(),
            "com.royhan.isladesk"
        );
    }

    fn cmd(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ringkasan_perintah_berhasil_gagal_dan_tidak_jalan() {
        let c = cmd(&["cargo", "build"]);
        assert_eq!(
            describe_run(&c, Some(0), Duration::from_secs(12)),
            "✓ cargo build selesai (12 dtk)"
        );
        assert_eq!(
            describe_run(&c, Some(0), Duration::from_secs(123)),
            "✓ cargo build selesai (2 mnt 3 dtk)"
        );
        assert_eq!(
            describe_run(&c, Some(101), Duration::from_secs(5)),
            "✕ cargo build gagal, kode 101 (5 dtk)"
        );
        assert_eq!(
            describe_run(&c, None, Duration::ZERO),
            "✕ cargo build tidak berjalan"
        );
    }

    #[test]
    fn nama_perintah_dipotong_tiga_kata() {
        let c = cmd(&["npm", "run", "build", "--", "--watch"]);
        assert!(describe_run(&c, Some(0), Duration::ZERO).starts_with("✓ npm run build selesai"));
    }
}
