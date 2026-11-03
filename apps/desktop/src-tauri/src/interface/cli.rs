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
    // shortcut: berkas bawaan hanya dikenal di Windows (%APPDATA%); platform lain pakai --ipc.
    let ipc_file = ipc_file.or_else(|| {
        std::env::var_os("APPDATA").map(|d| {
            PathBuf::from(d)
                .join("com.royhan.isladesk")
                .join("ipc.json")
        })
    });
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

/// Build rilis berjenis aplikasi GUI; tanpa ini `eprintln!` tidak terlihat di terminal pemanggil.
fn attach_parent_console() {
    #[cfg(windows)]
    // SAFETY: pemanggilan API Win32 tanpa pointer; gagal (tidak ada konsol induk) boleh diabaikan.
    unsafe {
        use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
