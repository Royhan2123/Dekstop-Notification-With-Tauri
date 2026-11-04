use std::sync::Arc;

use tauri::State;

use crate::application::github_service::GithubService;
use crate::interface::dto::RadarDto;

type Svc<'a> = State<'a, Arc<GithubService>>;

async fn blocking<T: Send + 'static>(
    svc: &Svc<'_>,
    f: impl FnOnce(&GithubService) -> Result<T, crate::domain::error::GithubError> + Send + 'static,
) -> Result<T, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || f(&svc))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn github_token_status(svc: Svc<'_>) -> Result<bool, String> {
    blocking(&svc, |s| s.has_token()).await
}

/// Token hanya masuk ke brankas OS; tidak ada command untuk membacanya kembali.
#[tauri::command]
#[specta::specta]
pub async fn github_token_set(svc: Svc<'_>, token: String) -> Result<(), String> {
    blocking(&svc, move |s| s.set_token(&token)).await
}

#[tauri::command]
#[specta::specta]
pub async fn github_token_clear(svc: Svc<'_>) -> Result<(), String> {
    blocking(&svc, |s| s.clear_token()).await
}

/// Keadaan GitHub terbaru (dipakai ulang bila belum semenit); galat dikembalikan sebagai teks.
#[tauri::command]
#[specta::specta]
pub async fn github_radar(svc: Svc<'_>) -> Result<RadarDto, String> {
    blocking(&svc, |s| s.radar()).await.map(|r| (&r).into())
}
