//! Satu-satunya tempat dependency injection: rakit adapter -> service -> state.
use std::sync::Arc;

use tauri::{App, Manager};

use crate::application::island_service::IslandService;
use crate::interface::events::TauriSnapshotSink;
use crate::interface::window::hit_test::{self, Hitbox};

pub fn assemble(app: &App) {
    let sink = Arc::new(TauriSnapshotSink(app.handle().clone()));
    let rt = tauri::async_runtime::handle().inner().clone();
    app.manage(IslandService::new(sink, rt));

    let hitbox = Arc::new(Hitbox::default());
    app.manage(hitbox.clone());
    match app.get_webview_window("island") {
        Some(window) => hit_test::spawn(window, hitbox),
        None => tracing::error!("jendela 'island' tidak ditemukan"),
    }
}
