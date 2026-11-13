pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interface;

#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri::WindowEvent;
use tauri_plugin_autostart::MacosLauncher;
use tauri_specta::{Builder, collect_commands, collect_events};

use interface::commands::ask::{
    ai_active, ai_models, ai_test, ask_clear, ask_history, ask_open, ask_submit,
};
use interface::commands::bridge::{bridge_pair, bridge_paired, bridge_set_rate, bridge_unpair};
use interface::commands::calendar::calendar_next;
use interface::commands::clipboard::{clipboard_clear, clipboard_copy, clipboard_list};
use interface::commands::disk::{disk_drives, disk_reveal, disk_scan, disk_status};
use interface::commands::github::{
    github_radar, github_token_clear, github_token_set, github_token_status,
};
use interface::commands::health::{health_peek, health_summary};
use interface::commands::island::{
    approval_decide, dev_simulate, island_click, island_dismiss, island_hitbox, island_hover,
    island_notice, island_set_dnd, island_show_media, island_snapshot, jarvis_ai_get,
    jarvis_ai_set, jarvis_ai_test,
};
use interface::commands::lyrics::lyrics_fetch;
use interface::commands::media::{media_control, media_cycle};
use interface::commands::net::net_rate;
use interface::commands::pomodoro::{pomodoro_start, pomodoro_status, pomodoro_stop};
use interface::commands::power::power_perform;
use interface::commands::review::{
    close_review, open_review, review_decide, review_unwatch, review_watch,
};
use interface::commands::settings::{
    ai_ready, api_key_clear, api_key_set, api_key_status, close_settings, open_settings,
    settings_get, settings_update,
};
use interface::commands::shelf::{
    shelf_add, shelf_clear, shelf_copy, shelf_open_file, shelf_remove, shelf_reveal, shelf_show,
};
use interface::commands::startup::{
    startup_add, startup_installed, startup_list, startup_move, startup_remove, startup_run,
    startup_set_delay,
};
use interface::commands::tasks::{
    tasks_add, tasks_clear_done, tasks_list, tasks_remove, tasks_set_done, tasks_set_note,
    tasks_update_note,
};
use interface::commands::theme::{theme_fetch, wallpaper_current};
use interface::dto::{IslandSnapshotEvent, SettingsChangedEvent};
use interface::window::{review_window, settings_window};

/// Argumen yang ditambahkan Windows saat membuka Isla Desk saat masuk (lihat plugin autostart).
pub const STARTUP_FLAG: &str = "--autostart";

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            island_snapshot,
            island_hover,
            island_click,
            island_dismiss,
            approval_decide,
            jarvis_ai_get,
            jarvis_ai_set,
            jarvis_ai_test,
            island_show_media,
            island_hitbox,
            island_set_dnd,
            island_notice,
            dev_simulate,
            media_control,
            media_cycle,
            settings_get,
            settings_update,
            api_key_status,
            ai_ready,
            api_key_set,
            api_key_clear,
            open_settings,
            close_settings,
            review_decide,
            review_watch,
            review_unwatch,
            open_review,
            close_review,
            health_summary,
            health_peek,
            net_rate,
            lyrics_fetch,
            calendar_next,
            github_token_status,
            github_token_set,
            github_token_clear,
            github_radar,
            theme_fetch,
            wallpaper_current,
            tasks_list,
            tasks_add,
            tasks_set_done,
            tasks_remove,
            tasks_clear_done,
            tasks_set_note,
            tasks_update_note,
            power_perform,
            startup_list,
            startup_installed,
            startup_add,
            startup_set_delay,
            startup_move,
            startup_remove,
            startup_run,
            disk_drives,
            disk_scan,
            disk_status,
            disk_reveal,
            bridge_pair,
            bridge_paired,
            bridge_unpair,
            bridge_set_rate,
            pomodoro_status,
            pomodoro_start,
            pomodoro_stop,
            clipboard_list,
            clipboard_copy,
            clipboard_clear,
            ask_open,
            ask_submit,
            ai_test,
            ai_models,
            ai_active,
            ask_history,
            ask_clear,
            shelf_add,
            shelf_show,
            shelf_remove,
            shelf_clear,
            shelf_open_file,
            shelf_reveal,
            shelf_copy
        ])
        .events(collect_events![IslandSnapshotEvent, SettingsChangedEvent])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().init();
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("gagal membuat bindings.ts");

    #[allow(unused_mut)]
    let mut app_builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    {
        app_builder = app_builder.plugin(tauri_nspanel::init());
    }
    app_builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Argumen ini membedakan start otomatis Windows dari peluncuran biasa (menjalankan daftar startup).
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![STARTUP_FLAG]),
        ))
        .invoke_handler(builder.invoke_handler())
        .on_window_event(|window, event| {
            // Menutup Setelan/Review hanya menyembunyikannya; aplikasi hidup di pulau.
            if let WindowEvent::CloseRequested { api, .. } = event
                && matches!(
                    window.label(),
                    settings_window::LABEL | review_window::LABEL
                )
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            builder.mount_events(app);
            // macOS: tanpa ikon Dock dan terlihat di semua Space (termasuk di atas app layar penuh),
            // kalau tidak pulau hanya muncul di Desktop biasa.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            bootstrap::assemble(app);
            #[cfg(target_os = "macos")]
            if let Some(island) = app.get_webview_window("island") {
                let _ = island.set_visible_on_all_workspaces(true);
                interface::window::island_window::float_over_fullscreen(&island);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
