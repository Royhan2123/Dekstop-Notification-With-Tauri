use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::application::task_service::TaskService;
use crate::interface::dto::TaskDto;

const OFF: &str = "daftar tugas nonaktif (database tidak bisa dibuka)";

fn service(app: &AppHandle) -> Result<Arc<TaskService>, String> {
    app.try_state::<Arc<TaskService>>()
        .map(|s| Arc::clone(s.inner()))
        .ok_or_else(|| OFF.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn tasks_list(app: AppHandle) -> Result<Vec<TaskDto>, String> {
    let tasks = service(&app)?.list().map_err(|e| e.to_string())?;
    Ok(tasks.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn tasks_add(app: AppHandle, title: String) -> Result<(), String> {
    service(&app)?.add(&title).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn tasks_set_done(app: AppHandle, id: i32, done: bool) -> Result<(), String> {
    service(&app)?.set_done(id, done).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn tasks_remove(app: AppHandle, id: i32) -> Result<(), String> {
    service(&app)?.remove(id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn tasks_clear_done(app: AppHandle) -> Result<(), String> {
    service(&app)?.clear_done().map_err(|e| e.to_string())
}
