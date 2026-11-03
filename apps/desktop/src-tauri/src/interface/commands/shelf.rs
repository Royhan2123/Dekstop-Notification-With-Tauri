use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::application::clipboard_service::ClipboardService;
use crate::application::island_service::IslandService;
use crate::domain::island::action::IslandAction;

type Island<'a> = State<'a, Arc<IslandService>>;

/// Berkas dari jendela diterima apa adanya; yang sudah tidak ada dibuang di sini (batas kepercayaan).
#[tauri::command]
#[specta::specta]
pub fn shelf_add(island: Island, paths: Vec<String>) {
    let paths: Vec<String> = paths
        .into_iter()
        .filter(|p| Path::new(p).exists())
        .collect();
    if !paths.is_empty() {
        island.dispatch(IslandAction::ShelfAdd(paths));
    }
}

#[tauri::command]
#[specta::specta]
pub fn shelf_show(island: Island) {
    island.dispatch(IslandAction::ShelfShow);
}

#[tauri::command]
#[specta::specta]
pub fn shelf_remove(island: Island, path: String) {
    island.dispatch(IslandAction::ShelfRemove(path));
}

#[tauri::command]
#[specta::specta]
pub fn shelf_clear(island: Island) {
    island.dispatch(IslandAction::ShelfClear);
}

/// Hanya berkas yang memang ada di rak yang boleh dibuka, ditunjukkan, atau disalin.
fn on_shelf(island: &IslandService, path: &str) -> bool {
    island.snapshot().shelf.iter().any(|p| p == path)
}

#[tauri::command]
#[specta::specta]
pub fn shelf_open_file(app: AppHandle, island: Island, path: String) -> Result<(), String> {
    if !on_shelf(&island, &path) {
        return Err("berkas tidak ada di rak".into());
    }
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn shelf_reveal(app: AppHandle, island: Island, path: String) -> Result<(), String> {
    if !on_shelf(&island, &path) {
        return Err("berkas tidak ada di rak".into());
    }
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())
}

/// Salin berkas ke clipboard supaya bisa ditempel (Ctrl+V) di folder atau aplikasi mana pun.
#[tauri::command]
#[specta::specta]
pub fn shelf_copy(
    island: Island,
    clipboard: State<'_, Arc<ClipboardService>>,
    path: String,
) -> bool {
    on_shelf(&island, &path) && clipboard.copy_files(&[path])
}
