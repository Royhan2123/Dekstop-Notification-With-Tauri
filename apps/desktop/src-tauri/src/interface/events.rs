use tauri::AppHandle;
use tauri_specta::Event;

use crate::domain::island::state::IslandState;
use crate::domain::ports::SnapshotSink;
use crate::interface::dto::IslandSnapshotEvent;

/// Mengirim snapshot state pulau ke frontend sebagai event Tauri.
pub struct TauriSnapshotSink(pub AppHandle);

impl SnapshotSink for TauriSnapshotSink {
    fn publish(&self, state: &IslandState) {
        if let Err(e) = IslandSnapshotEvent(state.into()).emit(&self.0) {
            tracing::warn!("gagal emit snapshot pulau: {e}");
        }
    }
}
