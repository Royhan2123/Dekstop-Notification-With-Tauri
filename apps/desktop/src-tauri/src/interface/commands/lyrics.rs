use std::sync::Arc;

use tauri::State;

use crate::application::lyrics_service::LyricsService;
use crate::interface::dto::LyricLineDto;

/// Lirik berwaktu untuk lagu yang sedang diputar; kosong bila tidak ada. Galat (server sibuk, tanpa
/// internet) dikembalikan sebagai teks supaya tampilan bisa mencoba lagi nanti.
#[tauri::command]
#[specta::specta]
pub async fn lyrics_fetch(
    svc: State<'_, Arc<LyricsService>>,
    artist: String,
    title: String,
    duration_ms: u32,
) -> Result<Vec<LyricLineDto>, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || svc.fetch(&artist, &title, duration_ms))
        .await
        .map_err(|e| e.to_string())?
        .map(|lines| {
            lines
                .into_iter()
                .map(|l| LyricLineDto {
                    ms: l.ms,
                    text: l.text,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}
