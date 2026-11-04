use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::application::settings_service::SettingsService;
use crate::domain::error::SettingsError;
use crate::domain::model::settings::{AiProvider, Settings};
use crate::interface::window::settings_window;

type Svc<'a> = State<'a, Arc<SettingsService>>;

/// Jalankan pekerjaan yang bisa memblokir (brankas OS, berkas) di luar thread UI.
async fn blocking<T: Send + 'static>(
    svc: &Svc<'_>,
    f: impl FnOnce(Arc<SettingsService>) -> Result<T, SettingsError> + Send + 'static,
) -> Result<T, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || f(svc))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn settings_get(svc: Svc) -> Settings {
    svc.get()
}

#[tauri::command]
#[specta::specta]
pub async fn settings_update(svc: Svc<'_>, settings: Settings) -> Result<Settings, String> {
    blocking(&svc, move |s| s.update(settings)).await
}

/// Penyedia AI yang siap dipilih dari kartu Tanya AI.
#[tauri::command]
#[specta::specta]
pub async fn ai_ready(svc: Svc<'_>) -> Result<Vec<AiProvider>, String> {
    blocking(&svc, |s| s.ready_providers()).await
}

#[tauri::command]
#[specta::specta]
pub async fn api_key_status(svc: Svc<'_>, provider: AiProvider) -> Result<bool, String> {
    blocking(&svc, move |s| s.has_api_key(provider)).await
}

/// API Key hanya masuk ke brankas OS; tidak ada command untuk membacanya kembali.
#[tauri::command]
#[specta::specta]
pub async fn api_key_set(svc: Svc<'_>, provider: AiProvider, key: String) -> Result<(), String> {
    blocking(&svc, move |s| s.set_api_key(provider, &key)).await
}

#[tauri::command]
#[specta::specta]
pub async fn api_key_clear(svc: Svc<'_>, provider: AiProvider) -> Result<(), String> {
    blocking(&svc, move |s| s.clear_api_key(provider)).await
}

#[tauri::command]
#[specta::specta]
pub fn open_settings(app: AppHandle) {
    settings_window::show(&app);
}

#[tauri::command]
#[specta::specta]
pub fn close_settings(app: AppHandle) {
    settings_window::hide(&app);
}
