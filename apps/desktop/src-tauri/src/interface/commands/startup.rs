use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::application::startup_service::StartupService;
use crate::interface::dto::{InstalledAppDto, StartupAppDto};

const OFF: &str = "daftar startup nonaktif (database tidak bisa dibuka)";

fn service(app: &AppHandle) -> Result<Arc<StartupService>, String> {
    app.try_state::<Arc<StartupService>>()
        .map(|s| Arc::clone(s.inner()))
        .ok_or_else(|| OFF.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn startup_list(app: AppHandle) -> Result<Vec<StartupAppDto>, String> {
    let apps = service(&app)?.list().map_err(|e| e.to_string())?;
    Ok(apps.iter().map(Into::into).collect())
}

/// Aplikasi terpasang (dari registri, bisa memakan ratusan ms) untuk dipilih di daftar.
#[tauri::command]
#[specta::specta]
pub async fn startup_installed(app: AppHandle) -> Result<Vec<InstalledAppDto>, String> {
    let svc = service(&app)?;
    let apps = tauri::async_runtime::spawn_blocking(move || svc.installed())
        .await
        .map_err(|e| e.to_string())?;
    Ok(apps.iter().map(Into::into).collect())
}

/// Tambahkan aplikasi pilihan pengguna ke akhir daftar startup.
#[tauri::command]
#[specta::specta]
pub fn startup_add(app: AppHandle, name: String, path: String) -> Result<(), String> {
    service(&app)?.add(&name, &path).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn startup_set_delay(app: AppHandle, id: i32, delay_secs: u32) -> Result<(), String> {
    service(&app)?
        .set_delay(id, delay_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn startup_move(app: AppHandle, id: i32, up: bool) -> Result<(), String> {
    service(&app)?.move_item(id, up).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn startup_remove(app: AppHandle, id: i32) -> Result<(), String> {
    service(&app)?.remove(id).map_err(|e| e.to_string())
}

/// Jalankan semua aplikasi sekarang (uji coba) di latar belakang, dengan jeda yang sama seperti saat startup.
#[tauri::command]
#[specta::specta]
pub fn startup_run(app: AppHandle) -> Result<(), String> {
    service(&app)?
        .run_all()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
