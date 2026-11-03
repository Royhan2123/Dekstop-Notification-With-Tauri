use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::application::clipboard_service::ClipboardService;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct ClipEntryDto {
    pub id: u32,
    pub text: String,
}

type Svc<'a> = State<'a, Arc<ClipboardService>>;

#[tauri::command]
#[specta::specta]
pub fn clipboard_list(svc: Svc, query: String) -> Vec<ClipEntryDto> {
    svc.list(&query)
        .into_iter()
        .map(|e| ClipEntryDto {
            id: e.id,
            text: e.text,
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn clipboard_copy(svc: Svc, id: u32) -> Result<(), String> {
    svc.copy(id)
        .then_some(())
        .ok_or_else(|| "item tidak ditemukan atau clipboard tidak bisa ditulis".to_string())
}

#[tauri::command]
#[specta::specta]
pub fn clipboard_clear(svc: Svc) {
    svc.clear();
}
