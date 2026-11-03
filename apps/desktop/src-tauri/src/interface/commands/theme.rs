use std::sync::Arc;

use tauri::State;

use crate::application::theme_service::ThemeService;

type Svc<'a> = State<'a, Arc<ThemeService>>;

/// Isi berkas tema dari sebuah tautan https; penerapannya lewat jalur impor berkas di Setelan.
#[tauri::command]
#[specta::specta]
pub async fn theme_fetch(svc: Svc<'_>, url: String) -> Result<String, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.fetch_theme(&url))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Wallpaper Windows saat ini (data URL JPEG kecil); galat bila tidak terbaca.
#[tauri::command]
#[specta::specta]
pub async fn wallpaper_current(svc: Svc<'_>) -> Result<String, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.wallpaper())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "wallpaper tidak terbaca".to_string())
}
