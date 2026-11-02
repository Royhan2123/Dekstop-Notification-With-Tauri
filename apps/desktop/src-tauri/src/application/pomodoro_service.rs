use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::runtime::Handle;

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::island::action::IslandAction;
use crate::domain::model::settings::Language;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PomodoroPhase {
    Idle,
    Focus,
    Break,
}

struct Run {
    phase: PomodoroPhase,
    /// Naik tiap fase baru atau berhenti; timer lama yang terlambat jadi tidak berlaku.
    generation: u64,
    ends: Instant,
}

/// Pomodoro: fokus (Jangan Ganggu menyala) lalu istirahat, tiap akhir fase muncul di pulau.
pub struct PomodoroService {
    island: Arc<IslandService>,
    settings: Arc<SettingsService>,
    rt: Handle,
    focus: Duration,
    rest: Duration,
    run: Mutex<Run>,
}

impl PomodoroService {
    pub fn new(
        island: Arc<IslandService>,
        settings: Arc<SettingsService>,
        rt: Handle,
        focus: Duration,
        rest: Duration,
    ) -> Arc<Self> {
        Arc::new(Self {
            island,
            settings,
            rt,
            focus,
            rest,
            run: Mutex::new(Run {
                phase: PomodoroPhase::Idle,
                generation: 0,
                ends: Instant::now(),
            }),
        })
    }

    pub fn start_focus(self: &Arc<Self>) {
        self.island.dispatch(IslandAction::DoNotDisturb(true));
        self.begin(PomodoroPhase::Focus, self.focus);
    }

    pub fn stop(&self) {
        let was = {
            let mut run = self.run();
            let was = run.phase;
            run.phase = PomodoroPhase::Idle;
            run.generation += 1;
            was
        };
        if was == PomodoroPhase::Focus {
            self.island.dispatch(IslandAction::DoNotDisturb(false));
        }
    }

    /// Fase sekarang dan sisa detiknya.
    pub fn status(&self) -> (PomodoroPhase, u64) {
        let run = self.run();
        match run.phase {
            PomodoroPhase::Idle => (PomodoroPhase::Idle, 0),
            phase => (
                phase,
                run.ends.saturating_duration_since(Instant::now()).as_secs(),
            ),
        }
    }

    fn begin(self: &Arc<Self>, phase: PomodoroPhase, duration: Duration) {
        let generation = {
            let mut run = self.run();
            run.phase = phase;
            run.generation += 1;
            run.ends = Instant::now() + duration;
            run.generation
        };
        let this = Arc::clone(self);
        self.rt.spawn(async move {
            tokio::time::sleep(duration).await;
            this.finish(generation);
        });
    }

    fn finish(self: &Arc<Self>, generation: u64) {
        let phase = {
            let run = self.run();
            if run.generation != generation {
                return;
            }
            run.phase
        };
        let id = self.settings.get().language == Language::Id;
        match phase {
            PomodoroPhase::Focus => {
                self.island.dispatch(IslandAction::DoNotDisturb(false));
                self.island.dispatch(IslandAction::Notice(
                    if id {
                        "Fokus selesai, saatnya istirahat"
                    } else {
                        "Focus done, time for a break"
                    }
                    .into(),
                ));
                self.begin(PomodoroPhase::Break, self.rest);
            }
            PomodoroPhase::Break => {
                self.run().phase = PomodoroPhase::Idle;
                self.island.dispatch(IslandAction::Notice(
                    if id {
                        "Istirahat selesai"
                    } else {
                        "Break over"
                    }
                    .into(),
                ));
            }
            PomodoroPhase::Idle => {}
        }
    }

    fn run(&self) -> std::sync::MutexGuard<'_, Run> {
        self.run.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::settings::Settings;

    fn fx() -> (
        Arc<PomodoroService>,
        Arc<IslandService>,
        tokio::runtime::Runtime,
    ) {
        let (settings, island, rt) = services(Settings::default());
        let hour = Duration::from_secs(3600);
        let svc = PomodoroService::new(island.clone(), settings, rt.handle().clone(), hour, hour);
        (svc, island, rt)
    }

    #[test]
    fn fokus_menyalakan_jangan_ganggu_lalu_istirahat_lalu_selesai() {
        let (svc, island, _rt) = fx();
        svc.start_focus();
        assert_eq!(svc.status().0, PomodoroPhase::Focus);
        assert!(island.snapshot().dnd);

        svc.finish(99); // timer usang
        assert_eq!(svc.status().0, PomodoroPhase::Focus);

        svc.finish(1);
        assert_eq!(svc.status().0, PomodoroPhase::Break);
        let s = island.snapshot();
        assert!(!s.dnd);
        assert!(matches!(s.view, View::Expanded(Content::Notice(_))));

        svc.finish(2);
        assert_eq!(svc.status(), (PomodoroPhase::Idle, 0));
    }

    #[test]
    fn berhenti_di_tengah_fokus_mematikan_jangan_ganggu_dan_membatalkan_timer() {
        let (svc, island, _rt) = fx();
        svc.start_focus();
        svc.stop();
        assert_eq!(svc.status().0, PomodoroPhase::Idle);
        assert!(!island.snapshot().dnd);
        svc.finish(1);
        assert_eq!(svc.status().0, PomodoroPhase::Idle);
        assert_eq!(island.snapshot().view, View::Idle);
    }
}
