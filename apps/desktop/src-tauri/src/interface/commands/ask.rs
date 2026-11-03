use std::sync::Arc;

use tauri::State;

use crate::application::quick_ask_service::QuickAskService;

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
