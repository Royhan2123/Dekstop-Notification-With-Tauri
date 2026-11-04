use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::application::disk_service::DiskService;
use crate::interface::dto::DiskStatusDto;

type Disk<'a> = State<'a, Arc<DiskService>>;

#[tauri::command]
#[specta::specta]
pub fn disk_drives(disk: Disk) -> Vec<String> {
    disk.drives()
}

/// Mulai memindai folder di latar belakang; hasilnya diambil lewat `disk_status`.
#[tauri::command]
#[specta::specta]
pub fn disk_scan(disk: Disk, path: String) -> Result<(), String> {
    if !Path::new(&path).is_dir() {
        return Err("folder tidak ditemukan".into());
    }
    disk.inner().start(&path);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn disk_status(disk: Disk) -> DiskStatusDto {
    (&disk.status()).into()
}

/// Tampilkan di Explorer: folder dibuka, berkas disorot di foldernya. Berkas tidak pernah dijalankan.
#[tauri::command]
#[specta::specta]
pub fn disk_reveal(app: AppHandle, path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.is_dir() {
        app.opener()
            .open_path(&path, None::<&str>)
            .map_err(|e| e.to_string())
    } else if p.exists() {
        app.opener()
            .reveal_item_in_dir(&path)
            .map_err(|e| e.to_string())
    } else {
        Err("sudah tidak ada".into())
    }
}
