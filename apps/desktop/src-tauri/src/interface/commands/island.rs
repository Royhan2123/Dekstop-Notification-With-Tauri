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
pub fn island_show_media(svc: Svc) {
    svc.inner().dispatch(IslandAction::MediaShow);
}

#[tauri::command]
#[specta::specta]
pub fn island_dismiss(svc: Svc) {
    svc.inner().dispatch(IslandAction::Dismissed);
}

/// Jawaban pengguna untuk kartu izin Jarvis: diteruskan ke API lokal Jarvis, lalu kartu ditutup.
#[tauri::command]
#[specta::specta]
pub async fn approval_decide(svc: Svc<'_>, id: String, allow: bool) -> Result<(), String> {
    let svc = svc.inner().clone();
    let sent = tauri::async_runtime::spawn_blocking(move || {
        crate::infrastructure::ai::jarvis::decide_approval(&id, allow)
    })
    .await
    .map_err(|e| e.to_string())?;
    svc.dispatch(IslandAction::Dismissed);
    sent.map_err(|e| e.to_string())
}

/// Setelan AI Jarvis sekarang (tanpa kunci; hanya `has_key`).
#[derive(serde::Serialize, specta::Type)]
pub struct JarvisAi {
    pub provider: String,
    pub providers: Vec<String>,
    pub api_base: String,
    pub fast_model: String,
    pub smart_model: String,
    pub has_key: bool,
    /// Suara Jarvis: mesin (fish/piper/system/edge) dan jenis (male/female). Hanya tampilan.
    pub voice_engine: String,
    pub voice_gender: String,
}

fn to_jarvis_ai(v: &serde_json::Value) -> JarvisAi {
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    JarvisAi {
        provider: s("provider"),
        providers: v
            .get("providers")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        api_base: s("api_base"),
        fast_model: s("fast_model"),
        smart_model: s("smart_model"),
        has_key: v.get("has_key").and_then(|x| x.as_bool()).unwrap_or(false),
        voice_engine: v
            .pointer("/voice/engine")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        voice_gender: v
            .pointer("/voice/gender")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn jarvis_ai_get() -> Result<JarvisAi, String> {
    tauri::async_runtime::spawn_blocking(crate::infrastructure::ai::jarvis::ai_get)
        .await
        .map_err(|e| e.to_string())?
        .map(|v| to_jarvis_ai(&v))
        .map_err(|e| e.to_string())
}

/// Ganti penyedia AI Jarvis (berlaku seketika). Memilih penyedia online = izin eksplisit pengguna.
#[tauri::command]
#[specta::specta]
pub async fn jarvis_ai_set(
    provider: String,
    api_key: Option<String>,
    api_base: Option<String>,
    fast_model: Option<String>,
) -> Result<JarvisAi, String> {
    let body = serde_json::json!({
        "provider": provider, "api_key": api_key, "api_base": api_base, "fast_model": fast_model,
    });
    tauri::async_runtime::spawn_blocking(move || crate::infrastructure::ai::jarvis::ai_set(&body))
        .await
        .map_err(|e| e.to_string())?
        .map(|v| to_jarvis_ai(&v))
        .map_err(|e| e.to_string())
}

/// Jarvis sudah dipasang di komputer ini (peluncurnya ada)?
#[tauri::command]
#[specta::specta]
pub fn jarvis_installed() -> bool {
    crate::infrastructure::ai::jarvis_runner::launcher().is_some()
}

/// Nyalakan Jarvis di latar. `"running"` bila sudah menyala, `"launched"` bila baru dijalankan.
#[tauri::command]
#[specta::specta]
pub async fn jarvis_start() -> Result<String, String> {
    use crate::infrastructure::ai::jarvis_runner::{Started, start};
    tauri::async_runtime::spawn_blocking(start)
        .await
        .map_err(|e| e.to_string())?
        .map(|s| match s {
            Started::AlreadyRunning => "running".to_string(),
            Started::Launched => "launched".to_string(),
        })
}

#[tauri::command]
#[specta::specta]
pub async fn jarvis_ai_test() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(crate::infrastructure::ai::jarvis::ai_test)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn island_set_dnd(svc: Svc, on: bool) {
    svc.inner().dispatch(IslandAction::DoNotDisturb(on));
}

/// Pemberitahuan singkat dari frontend (mis. pengingat sehat); otomatis tertutup, diam saat Jangan Ganggu.
#[tauri::command]
#[specta::specta]
pub fn island_notice(svc: Svc, text: String) {
    let text: String = text.chars().take(120).collect();
    if !text.trim().is_empty() {
        svc.inner().dispatch(IslandAction::Notice(text));
    }
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
            playing: true,
            position_ms: 30_000,
            duration_ms: 180_000,
            rate: 1.0,
            color: Some("#7c5cff".into()),
            session_count: 1,
            ..Default::default()
        })),
        // Lagu nyata yang punya lirik berwaktu di LRCLIB, untuk mencoba tampilan lirik.
        "media_lyrics" => IslandAction::MediaChanged(Some(MediaInfo {
            title: "Blinding Lights".into(),
            artist: "The Weeknd".into(),
            playing: true,
            position_ms: 30_000,
            duration_ms: 200_000,
            rate: 1.0,
            color: Some("#e0245e".into()),
            session_count: 1,
            ..Default::default()
        })),
        "media_off" => IslandAction::MediaChanged(None),
        "review" => IslandAction::ReviewStarted(ReviewId(1)),
        "review_done" => IslandAction::ReviewFinished(
            ReviewId(1),
            ReviewReport {
                summary: "Tidak ada masalah".into(),
                ..Default::default()
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
