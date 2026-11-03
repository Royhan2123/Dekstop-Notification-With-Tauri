pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interface;

use tauri::WindowEvent;
use tauri_plugin_autostart::MacosLauncher;
use tauri_specta::{Builder, collect_commands, collect_events};

use interface::commands::ask::{ask_open, ask_submit};
use interface::commands::bridge::{
    bridge_pair, bridge_paired, bridge_search, bridge_set_rate, bridge_unpair,
};
use interface::commands::clipboard::{clipboard_clear, clipboard_copy, clipboard_list};
use interface::commands::health::{health_peek, health_summary};
use interface::commands::island::{
    dev_simulate, island_click, island_dismiss, island_hitbox, island_hover, island_set_dnd,
    island_snapshot,
};
use interface::commands::lyrics::lyrics_fetch;
use interface::commands::media::{media_control, media_cycle};
use interface::commands::net::net_rate;
use interface::commands::pomodoro::{pomodoro_start, pomodoro_status, pomodoro_stop};
use interface::commands::review::{
    close_review, open_review, review_decide, review_unwatch, review_watch,
};
use interface::commands::settings::{
    api_key_clear, api_key_set, api_key_status, close_settings, open_settings, settings_get,
    settings_update,
};
use interface::commands::shelf::{
    shelf_add, shelf_clear, shelf_copy, shelf_open_file, shelf_remove, shelf_reveal, shelf_show,
};
use interface::commands::tasks::{
    tasks_add, tasks_clear_done, tasks_list, tasks_remove, tasks_set_done,
};
use interface::dto::{IslandSnapshotEvent, SettingsChangedEvent};
use interface::window::{review_window, settings_window};

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            island_snapshot,
            island_hover,
            island_click,
            island_dismiss,
            island_hitbox,
            island_set_dnd,
            dev_simulate,
            media_control,
            media_cycle,
            settings_get,
            settings_update,
            api_key_status,
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
            tasks_list,
            tasks_add,
            tasks_set_done,
            tasks_remove,
            tasks_clear_done,
            bridge_pair,
            bridge_paired,
            bridge_unpair,
            bridge_set_rate,
            bridge_search,
            pomodoro_status,
            pomodoro_start,
            pomodoro_stop,
            clipboard_list,
            clipboard_copy,
            clipboard_clear,
            ask_open,
            ask_submit,
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
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
            bootstrap::assemble(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
