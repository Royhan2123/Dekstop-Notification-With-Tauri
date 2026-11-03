//! Satu-satunya tempat dependency injection: rakit adapter -> service -> state.
use std::sync::Arc;
use std::time::Duration;

use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::application::bridge_service::BridgeService;
use crate::application::charger_service::ChargerService;
use crate::application::clipboard_service::ClipboardService;
use crate::application::fullscreen_service::FullscreenService;
use crate::application::health_service::HealthService;
use crate::application::island_service::IslandService;
use crate::application::media_service::MediaService;
use crate::application::net_service::NetService;
use crate::application::pomodoro_service::PomodoroService;
use crate::application::quick_ask_service::QuickAskService;
use crate::application::review::service::ReviewService;
use crate::application::settings_service::SettingsService;
use crate::application::task_service::TaskService;
use crate::application::volume_service::VolumeService;
use crate::domain::bridge::clean_query;
use crate::domain::island::action::IslandAction;
use crate::domain::model::review::PushRef;
use crate::domain::ports::SecretStore;
use crate::infrastructure::ai::DefaultAiFactory;
use crate::infrastructure::bridge::ws_server::WsBridge;
use crate::infrastructure::git::cli_git::CliGit;
use crate::infrastructure::git::hook::GitHookInstaller;
use crate::infrastructure::media::default_source;
use crate::infrastructure::security::regex_scanner::RegexScanner;
use crate::infrastructure::storage::keyring_store::KeyringStore;
use crate::infrastructure::storage::settings_store::JsonSettingsStore;
use crate::infrastructure::storage::sqlite::Database;
use crate::infrastructure::storage::task_store::SqliteTaskStore;
use crate::infrastructure::storage::usage_store::SqliteUsageStore;
use crate::infrastructure::system::autostart::TauriAutostart;
use crate::infrastructure::system::brightness::SystemBrightness;
use crate::infrastructure::system::clipboard::SystemClipboard;
use crate::infrastructure::system::fullscreen::SystemFullscreen;
use crate::infrastructure::system::idle::SystemIdle;
use crate::infrastructure::system::network::SystemNet;
use crate::infrastructure::system::power::SystemPower;
use crate::infrastructure::system::volume::SystemVolume;
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

    app.manage(PomodoroService::new(
        island.clone(),
        settings.clone(),
        tauri::async_runtime::handle().inner().clone(),
    ));

    let ws = WsBridge::new();
    let bridge = BridgeService::new(
        ws.clone(),
        secrets.clone(),
        island.clone(),
        settings.clone(),
    );
    app.manage(bridge.clone());
    match ws.start(bridge) {
        Ok(port) => tracing::info!("Isla Link mendengarkan di 127.0.0.1:{port}"),
        Err(e) => tracing::error!("gagal membuka server Isla Link: {e}"),
    }

    // Tanpa database, fitur riwayat (kesehatan, review) nonaktif; pulau tetap jalan.
    match Database::open(&data_dir.join("isla.db")) {
        Ok(db) => {
            let db = Arc::new(db);
            app.manage(db.clone());
            app.manage(TaskService::new(Arc::new(SqliteTaskStore(db.clone()))));
            let health = HealthService::new(
                Arc::new(SystemIdle),
                Arc::new(SystemBrightness),
                Arc::new(SqliteUsageStore(db)),
                settings.clone(),
                island.clone(),
            );
            app.manage(health.clone());
            spawn_health_ticker(health);
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
        Arc::new(DefaultAiFactory::new(secrets.clone())),
        settings.clone(),
        island.clone(),
        Duration::from_secs(120),
    );
    app.manage(review.clone());
    let ask = QuickAskService::new(
        Arc::new(DefaultAiFactory::new(secrets.clone())),
        settings.clone(),
        island.clone(),
    );
    app.manage(ask.clone());
    register_ask_shortcut(app, ask);

    let on_push = Arc::new(move |repo: &str, refs: &[PushRef]| review.review_push(repo, refs));
    let notice_island = island.clone();
    let on_notify: ipc::Notifier = Arc::new(move |text| {
        if let Some(text) = clean_query(text) {
            notice_island.dispatch(IslandAction::Notice(text));
        }
    });
    if let Err(e) = ipc::serve(&ipc_file, on_push, on_notify) {
        tracing::error!("gagal membuka jalur IPC hook: {e}");
    }

    spawn_charger_watch(ChargerService::new(
        Arc::new(SystemPower),
        settings.clone(),
        island.clone(),
    ));

    app.manage(NetService::new(Arc::new(SystemNet)));

    spawn_fullscreen_watch(FullscreenService::new(
        Arc::new(SystemFullscreen),
        settings.clone(),
        island.clone(),
    ));

    spawn_volume_watch(VolumeService::new(Arc::new(SystemVolume), island.clone()));

    let clipboard = ClipboardService::new(Arc::new(SystemClipboard), settings.clone());
    app.manage(clipboard.clone());
    spawn_clipboard_watch(clipboard);

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

/// Ctrl+Space membuka Quick Ask dan memberi fokus keyboard ke jendela pulau.
fn register_ask_shortcut(app: &App, ask: Arc<QuickAskService>) {
    let handle = app.handle().clone();
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
    let result = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_, _, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            ask.open();
            // Windows kadang menahan perpindahan fokus sesaat; coba lagi sampai jendela benar-benar fokus.
            if let Some(window) = handle.get_webview_window("island") {
                std::thread::spawn(move || {
                    for _ in 0..5 {
                        let _ = window.set_focus();
                        std::thread::sleep(Duration::from_millis(100));
                        if window.is_focused().unwrap_or(false) {
                            break;
                        }
                    }
                });
            }
        });
    if let Err(e) = result {
        // Mis. dipakai aplikasi lain; pulau tetap jalan tanpa Quick Ask.
        tracing::error!("gagal mendaftarkan Ctrl+Space: {e}");
    }
}

/// Periksa colokan charger tiap 3 detik; murah (satu panggilan API) jadi tidak perlu event OS.
fn spawn_charger_watch(charger: Arc<ChargerService>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            charger.tick();
        }
    });
}

/// Periksa aplikasi layar penuh tiap 2 detik; satu panggilan Shell API murah.
fn spawn_fullscreen_watch(fullscreen: Arc<FullscreenService>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            fullscreen.tick();
        }
    });
}

/// Periksa volume 4 kali per detik di thread sendiri (COM diinisialisasi per thread);
/// satu panggilan Core Audio murah, jadi tidak perlu notifikasi event OS.
fn spawn_volume_watch(volume: Arc<VolumeService>) {
    std::thread::spawn(move || {
        loop {
            volume.tick();
            std::thread::sleep(Duration::from_millis(250));
        }
    });
}

/// Periksa clipboard tiap detik; tanpa izin di setelan, layanan hanya membuang riwayat.
fn spawn_clipboard_watch(clipboard: Arc<ClipboardService>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let svc = clipboard.clone();
            let _ = tauri::async_runtime::spawn_blocking(move || svc.tick()).await;
        }
    });
}

/// Timer pencatatan: satu ketukan per menit untuk pencatatan dan pengingat.
fn spawn_health_ticker(health: Arc<HealthService>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let svc = health.clone();
            let _ = tauri::async_runtime::spawn_blocking(move || {
                svc.tick(chrono::Local::now().naive_local());
            })
            .await;
        }
    });
}
