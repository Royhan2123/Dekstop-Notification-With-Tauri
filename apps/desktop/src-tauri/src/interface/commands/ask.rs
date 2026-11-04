use std::sync::Arc;

use tauri::State;

use crate::application::quick_ask_service::QuickAskService;
use crate::interface::dto::AiActiveDto;

type Svc<'a> = State<'a, Arc<QuickAskService>>;

#[tauri::command]
#[specta::specta]
pub fn ask_open(svc: Svc) {
    svc.open();
}

#[tauri::command]
#[specta::specta]
pub fn ask_submit(svc: Svc, question: String) {
    svc.inner().submit(&question);
}

/// Penyedia dan model yang sedang dipakai Tanya AI, untuk pemilih di kartu jawaban.
#[tauri::command]
#[specta::specta]
pub fn ai_active(svc: Svc) -> AiActiveDto {
    let (provider, model) = svc.active();
    AiActiveDto { provider, model }
}

/// Daftar model dari penyedia terpilih, untuk menu pilihan model di Setelan.
#[tauri::command]
#[specta::specta]
pub async fn ai_models(svc: Svc<'_>) -> Result<Vec<String>, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.models())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Uji koneksi ke otak AI yang sedang dipilih; membalas teks singkat dari model bila berhasil.
#[tauri::command]
#[specta::specta]
pub async fn ai_test(svc: Svc<'_>) -> Result<String, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.test_connection())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
