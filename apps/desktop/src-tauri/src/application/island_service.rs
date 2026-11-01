use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::runtime::Handle;

use crate::domain::island::action::{Effect, IslandAction};
use crate::domain::island::reducer::reduce;
use crate::domain::island::state::{Ctx, IslandState};
use crate::domain::ports::SnapshotSink;

/// Pemegang state pulau: menjalankan reducer, menjadwalkan timer, dan mempublikasikan snapshot.
pub struct IslandService {
    state: Mutex<IslandState>,
    ctx: Ctx,
    sink: Arc<dyn SnapshotSink>,
    rt: Handle,
}

impl IslandService {
    pub fn new(sink: Arc<dyn SnapshotSink>, rt: Handle) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(IslandState::default()),
            ctx: Ctx::default(),
            sink,
            rt,
        })
    }

    pub fn snapshot(&self) -> IslandState {
        self.lock().clone()
    }

    pub fn dispatch(self: &Arc<Self>, action: IslandAction) {
        let effects = {
            let mut guard = self.lock();
            let (next, effects) = reduce(&guard, action, &self.ctx);
            if next != *guard {
                *guard = next;
                self.sink.publish(&guard);
            }
            effects
        };
        for effect in effects {
            let Effect::StartTimer { token, ms } = effect;
            let this = Arc::clone(self);
            self.rt.spawn(async move {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                this.dispatch(IslandAction::Timeout(token));
            });
        }
    }

    // Mutex yang keracunan tetap dipakai: state pulau selalu valid antar-aksi.
    fn lock(&self) -> MutexGuard<'_, IslandState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::HealthAlert;

    #[derive(Default)]
    struct CountingSink(Mutex<usize>);
    impl SnapshotSink for CountingSink {
        fn publish(&self, _: &IslandState) {
            *self.0.lock().unwrap() += 1;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn alert_otomatis_tertutup_oleh_timer_dan_snapshot_hanya_saat_berubah() {
        let sink = Arc::new(CountingSink::default());
        let svc = IslandService::new(sink.clone(), Handle::current());
        svc.dispatch(IslandAction::HealthAlert(HealthAlert {
            message: "x".into(),
        }));
        svc.dispatch(IslandAction::Hover(false)); // tidak mengubah state: tidak ada snapshot
        assert!(matches!(
            svc.snapshot().view,
            View::Expanded(Content::HealthAlert(_))
        ));
        tokio::time::sleep(Duration::from_millis(8_100)).await;
        assert_eq!(svc.snapshot().view, View::Idle);
        assert_eq!(*sink.0.lock().unwrap(), 2);
    }
}
