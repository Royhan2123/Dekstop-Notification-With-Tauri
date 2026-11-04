use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::application::power_service::PowerService;
use crate::interface::dto::PowerActionDto;

/// Dipanggil setelah pengguna menyetujui kartu konfirmasi (kecuali kunci layar).
#[tauri::command]
#[specta::specta]
pub fn power_perform(app: AppHandle, action: PowerActionDto) -> Result<(), String> {
    let svc = app
        .try_state::<Arc<PowerService>>()
        .map(|s| Arc::clone(s.inner()))
        .ok_or_else(|| "layanan daya tidak tersedia".to_string())?;
    svc.perform(action.into()).map_err(|e| e.to_string())
}
