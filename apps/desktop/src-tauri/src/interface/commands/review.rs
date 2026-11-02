use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::application::review::service::ReviewService;
use crate::domain::model::review::Decision;
use crate::interface::window::review_window;

type Svc<'a> = State<'a, Arc<ReviewService>>;

/// Jawaban pengguna pada kartu review (Perbaiki, Lanjut, Batal, setujui/tolak patch).
#[tauri::command]
#[specta::specta]
pub fn review_decide(svc: Svc, decision: Decision) {
    svc.decide(decision);
}

/// Awasi repo: pasang hook `pre-push`. Memanggil git, jadi dijalankan di luar thread UI.
#[tauri::command]
#[specta::specta]
pub async fn review_watch(svc: Svc<'_>, path: String) -> Result<(), String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.watch(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn review_unwatch(svc: Svc<'_>, path: String) -> Result<(), String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.unwatch(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn open_review(app: AppHandle) {
    review_window::show(&app);
}

#[tauri::command]
#[specta::specta]
pub fn close_review(app: AppHandle) {
    review_window::hide(&app);
}
