use std::sync::Arc;

use tauri::State;

use crate::application::island_service::IslandService;
use crate::domain::island::action::IslandAction;
use crate::domain::model::{HealthAlert, MediaInfo, ReviewId, ReviewReport};
use crate::interface::dto::IslandSnapshot;
use crate::interface::window::hit_test::{Hitbox, Rect};

type Svc<'a> = State<'a, Arc<IslandService>>;

#[tauri::command]
#[specta::specta]
pub fn island_snapshot(svc: Svc) -> IslandSnapshot {
    (&svc.snapshot()).into()
}

#[tauri::command]
#[specta::specta]
pub fn island_hover(svc: Svc, hovered: bool) {
    svc.inner().dispatch(IslandAction::Hover(hovered));
}

#[tauri::command]
#[specta::specta]
pub fn island_click(svc: Svc) {
    svc.inner().dispatch(IslandAction::Clicked);
}

#[tauri::command]
#[specta::specta]
pub fn island_dismiss(svc: Svc) {
    svc.inner().dispatch(IslandAction::Dismissed);
}

/// Hitbox pulau saat ini (piksel logis relatif ke jendela), dipakai untuk klik tembus.
#[tauri::command]
#[specta::specta]
pub fn island_hitbox(hitbox: State<'_, Arc<Hitbox>>, rect: Rect) {
    hitbox.set(rect);
}

/// Simulasi kejadian untuk mencoba pulau sebelum fitur aslinya ada. Hanya aktif di build debug.
#[tauri::command]
#[specta::specta]
pub fn dev_simulate(svc: Svc, kind: String) {
    if !cfg!(debug_assertions) {
        return;
    }
    let action = match kind.as_str() {
        "media" => IslandAction::MediaChanged(Some(MediaInfo {
            title: "Lagu contoh".into(),
            artist: "Artis contoh".into(),
        })),
        "media_off" => IslandAction::MediaChanged(None),
        "review" => IslandAction::ReviewStarted(ReviewId(1)),
        "review_done" => IslandAction::ReviewFinished(
            ReviewId(1),
            ReviewReport {
                summary: "Tidak ada masalah".into(),
            },
        ),
        "health" => IslandAction::HealthAlert(HealthAlert {
            message: "Istirahat dulu ya".into(),
        }),
        "notice" => IslandAction::Notice("Halo dari Isla".into()),
        "dismiss" => IslandAction::Dismissed,
        _ => return,
    };
    svc.inner().dispatch(action);
}
