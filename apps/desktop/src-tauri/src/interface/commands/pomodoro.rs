use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::application::pomodoro_service::{PomodoroPhase, PomodoroService};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroPhaseDto {
    Idle,
    Focus,
    Break,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Type)]
pub struct PomodoroStatusDto {
    pub phase: PomodoroPhaseDto,
    pub remaining_secs: u32,
}

type Svc<'a> = State<'a, Arc<PomodoroService>>;

#[tauri::command]
#[specta::specta]
pub fn pomodoro_status(svc: Svc) -> PomodoroStatusDto {
    let (phase, secs) = svc.status();
    PomodoroStatusDto {
        phase: match phase {
            PomodoroPhase::Idle => PomodoroPhaseDto::Idle,
            PomodoroPhase::Focus => PomodoroPhaseDto::Focus,
            PomodoroPhase::Break => PomodoroPhaseDto::Break,
        },
        remaining_secs: u32::try_from(secs).unwrap_or(u32::MAX),
    }
}

#[tauri::command]
#[specta::specta]
pub fn pomodoro_start(svc: Svc) {
    svc.inner().start_focus();
}

#[tauri::command]
#[specta::specta]
pub fn pomodoro_stop(svc: Svc) {
    svc.stop();
}
