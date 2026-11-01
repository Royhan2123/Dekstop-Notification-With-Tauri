pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interface;

use tauri_specta::{Builder, collect_commands, collect_events};

use interface::commands::island::{
    dev_simulate, island_click, island_dismiss, island_hitbox, island_hover, island_snapshot,
};
use interface::dto::IslandSnapshotEvent;

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            island_snapshot,
            island_hover,
            island_click,
            island_dismiss,
            island_hitbox,
            dev_simulate
        ])
        .events(collect_events![IslandSnapshotEvent])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            bootstrap::assemble(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
