//! Satu-satunya tempat dependency injection: rakit adapter -> service -> state.
use std::sync::Arc;
use std::time::Duration;

use tauri::{App, Manager};

use crate::application::island_service::IslandService;
use crate::application::media_service::MediaService;
use crate::application::review::service::ReviewService;
use crate::application::settings_service::SettingsService;
use crate::domain::model::review::PushRef;
use crate::domain::ports::SecretStore;
use crate::infrastructure::ai::DefaultAiFactory;
use crate::infrastructure::git::cli_git::CliGit;
use crate::infrastructure::git::hook::GitHookInstaller;
use crate::infrastructure::media::default_source;
use crate::infrastructure::security::regex_scanner::RegexScanner;
use crate::infrastructure::storage::keyring_store::KeyringStore;
use crate::infrastructure::storage::settings_store::JsonSettingsStore;
use crate::infrastructure::storage::sqlite::Database;
use crate::infrastructure::system::autostart::TauriAutostart;
use crate::interface::events::{TauriSettingsSink, TauriSnapshotSink};
use crate::interface::ipc;
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

    let secrets: Arc<dyn SecretStore> = Arc::new(KeyringStore::new("isla-desk"));
    let settings = SettingsService::new(
        Arc::new(JsonSettingsStore::new(data_dir.join("settings.json"))),
        secrets.clone(),
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

    // Batas waktu menunggu keputusan di kartu; setelah itu berlaku setelan "saat waktu habis".
    let ipc_file = data_dir.join("ipc.json");
    let exe = std::env::current_exe().unwrap_or_default();
    let review = ReviewService::new(
        Arc::new(CliGit),
        Arc::new(GitHookInstaller::new(exe, ipc_file.clone())),
        Arc::new(RegexScanner),
        Arc::new(DefaultAiFactory::new(secrets)),
        settings.clone(),
        island.clone(),
        Duration::from_secs(120),
    );
    app.manage(review.clone());
    let on_push = Arc::new(move |repo: &str, refs: &[PushRef]| review.review_push(repo, refs));
    if let Err(e) = ipc::serve(&ipc_file, on_push) {
        tracing::error!("gagal membuka jalur IPC hook: {e}");
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
