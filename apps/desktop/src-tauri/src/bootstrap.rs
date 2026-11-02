//! Satu-satunya tempat dependency injection: rakit adapter -> service -> state.
use std::sync::Arc;

use tauri::{App, Manager};

use crate::application::island_service::IslandService;
use crate::application::media_service::MediaService;
use crate::application::settings_service::SettingsService;
use crate::infrastructure::media::default_source;
use crate::infrastructure::storage::keyring_store::KeyringStore;
use crate::infrastructure::storage::settings_store::JsonSettingsStore;
use crate::infrastructure::storage::sqlite::Database;
use crate::infrastructure::system::autostart::TauriAutostart;
use crate::interface::events::{TauriSettingsSink, TauriSnapshotSink};
use crate::interface::window::hit_test::{self, Hitbox};
use crate::interface::window::settings_window;

pub fn assemble(app: &App) {
    let handle = app.handle().clone();
    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("isla-desk"));

    let rt = tauri::async_runtime::handle().inner().clone();
    let island = IslandService::new(Arc::new(TauriSnapshotSink(handle.clone())), rt);
    app.manage(island.clone());

    let settings = SettingsService::new(
        Arc::new(JsonSettingsStore::new(data_dir.join("settings.json"))),
        Arc::new(KeyringStore::new("isla-desk")),
        Arc::new(TauriAutostart(handle.clone())),
        Arc::new(TauriSettingsSink(handle.clone())),
        island.clone(),
    );
    app.manage(settings.clone());

    // Tanpa database, fitur riwayat (kesehatan, review) nonaktif; pulau tetap jalan.
    match Database::open(&data_dir.join("isla.db")) {
        Ok(db) => {
            app.manage(Arc::new(db));
        }
        Err(e) => tracing::error!("gagal membuka database: {e}"),
    }

    let media = MediaService::new(default_source(), island);
    app.manage(media.clone());
    media.start();

    let hitbox = Arc::new(Hitbox::default());
    app.manage(hitbox.clone());
    match app.get_webview_window("island") {
        Some(window) => hit_test::spawn(window, hitbox),
        None => tracing::error!("jendela 'island' tidak ditemukan"),
    }

    if !settings.get().onboarded {
        settings_window::show(&handle);
    }
}
