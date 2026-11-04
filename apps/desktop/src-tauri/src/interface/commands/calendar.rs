use std::sync::Arc;

use chrono::{Local, Offset};
use tauri::State;

use crate::application::calendar_service::CalendarService;
use crate::interface::dto::MeetingDto;

/// Rapat berikutnya dari kalender di setelan; kosong bila tautan belum diisi atau tidak ada rapat.
/// Galat (tautan salah, tanpa internet) dikembalikan sebagai teks untuk ditampilkan di Setelan.
#[tauri::command]
#[specta::specta]
pub async fn calendar_next(
    svc: State<'_, Arc<CalendarService>>,
) -> Result<Option<MeetingDto>, String> {
    let svc = Arc::clone(svc.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let now = Local::now();
        svc.next(now.naive_local(), now.offset().fix().local_minus_utc())
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|m| {
        m.map(|m| MeetingDto {
            title: m.title,
            time: m.start.format("%H:%M").to_string(),
            date: m.start.format("%Y-%m-%d").to_string(),
        })
    })
    .map_err(|e| e.to_string())
}
