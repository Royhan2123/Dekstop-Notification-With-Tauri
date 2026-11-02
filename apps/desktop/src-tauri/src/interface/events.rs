use tauri::AppHandle;
use tauri_specta::Event;

use crate::domain::island::state::IslandState;
use crate::domain::model::settings::Settings;
use crate::domain::ports::{SettingsSink, SnapshotSink};
use crate::interface::dto::{IslandSnapshotEvent, SettingsChangedEvent};

/// Mengirim snapshot state pulau ke frontend sebagai event Tauri.
pub struct TauriSnapshotSink(pub AppHandle);

impl SnapshotSink for TauriSnapshotSink {
    fn publish(&self, state: &IslandState) {
        if let Err(e) = IslandSnapshotEvent(state.into()).emit(&self.0) {
            tracing::warn!("gagal emit snapshot pulau: {e}");
        }
    }
}

/// Mengirim setelan terbaru ke semua jendela.
pub struct TauriSettingsSink(pub AppHandle);

impl SettingsSink for TauriSettingsSink {
    fn publish(&self, settings: &Settings) {
        if let Err(e) = SettingsChangedEvent(settings.clone()).emit(&self.0) {
            tracing::warn!("gagal emit setelan: {e}");
        }
    }
}
