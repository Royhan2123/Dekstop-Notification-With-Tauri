use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::application::health_service::HealthService;
use crate::interface::dto::HealthSummaryDto;

const OFF: &str = "fitur kesehatan nonaktif (database tidak bisa dibuka)";

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

fn service(app: &AppHandle) -> Result<Arc<HealthService>, String> {
    app.try_state::<Arc<HealthService>>()
        .map(|s| Arc::clone(s.inner()))
        .ok_or_else(|| OFF.to_string())
}

/// Ringkasan hari ini dan tujuh hari terakhir untuk grafik di Setelan.
#[tauri::command]
#[specta::specta]
pub async fn health_summary(app: AppHandle) -> Result<HealthSummaryDto, String> {
    let svc = service(&app)?;
    tauri::async_runtime::spawn_blocking(move || svc.summary(now()))
        .await
        .map_err(|e| e.to_string())?
        .map(|s| (&s).into())
        .map_err(|e| e.to_string())
}

/// Klik pulau saat idle: tampilkan ringkasan hari ini.
#[tauri::command]
#[specta::specta]
pub async fn health_peek(app: AppHandle) -> Result<(), String> {
    let svc = service(&app)?;
    tauri::async_runtime::spawn_blocking(move || svc.peek(now()))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
