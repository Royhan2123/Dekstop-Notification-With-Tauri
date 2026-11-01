//! Port: kemampuan yang dibutuhkan domain/application dan diimplementasikan di luar.
use crate::domain::island::state::IslandState;

/// Penerima snapshot state pulau (mis. emit event ke frontend).
pub trait SnapshotSink: Send + Sync {
    fn publish(&self, state: &IslandState);
}
