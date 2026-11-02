use std::sync::Arc;

use tauri::State;

use crate::application::media_service::MediaService;
use crate::interface::dto::MediaCommandDto;

#[tauri::command]
#[specta::specta]
pub async fn media_control(
    svc: State<'_, Arc<MediaService>>,
    cmd: MediaCommandDto,
) -> Result<(), String> {
    let svc = Arc::clone(svc.inner());
    // Kontrol memblokir sampai OS menjawab, jadi jangan di thread async/UI.
    tauri::async_runtime::spawn_blocking(move || svc.control(cmd.into()))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Pindah sesi media (`delta` > 0 berikutnya, < 0 sebelumnya).
#[tauri::command]
#[specta::specta]
pub fn media_cycle(svc: State<'_, Arc<MediaService>>, delta: i32) {
    svc.cycle(delta);
}
