use std::sync::Arc;

use tauri::State;

use crate::application::bridge_service::BridgeService;

type Svc<'a> = State<'a, Arc<BridgeService>>;

/// Buat kode pairing baru; kodenya juga tampil di pulau.
#[tauri::command]
#[specta::specta]
pub fn bridge_pair(svc: Svc) -> String {
    svc.begin_pairing()
}

#[tauri::command]
#[specta::specta]
pub fn bridge_paired(svc: Svc) -> Result<bool, String> {
    svc.is_paired().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn bridge_unpair(svc: Svc) -> Result<(), String> {
    svc.unpair().map_err(|e| e.to_string())
}

/// Kecepatan putar 0,25x-2x untuk video/musik di browser.
#[tauri::command]
#[specta::specta]
pub fn bridge_set_rate(svc: Svc, rate: f64) -> Result<(), String> {
    svc.set_rate(rate).map_err(|e| e.to_string())
}
