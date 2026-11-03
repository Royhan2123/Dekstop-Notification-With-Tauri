use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::runtime::Handle;

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::island::action::IslandAction;
use crate::domain::model::settings::{Language, Settings};
use crate::domain::ports::SoundPlayer;

/// Batas durasi satu sesi fokus.
const MIN_FOCUS: Duration = Duration::from_secs(1);
const MAX_FOCUS: Duration = Duration::from_secs(24 * 3600);

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

/// Pomodoro: fokus lalu istirahat. Tiap akhir fase muncul di pulau dan berbunyi.
/// Tidak menyentuh Jangan Ganggu; itu pilihan terpisah milik pengguna.
pub struct PomodoroService {
    island: Arc<IslandService>,
    settings: Arc<SettingsService>,
    sound: Arc<dyn SoundPlayer>,
    rt: Handle,
    run: Mutex<Run>,
}

impl PomodoroService {
    pub fn new(
        island: Arc<IslandService>,
        settings: Arc<SettingsService>,
        sound: Arc<dyn SoundPlayer>,
        rt: Handle,
    ) -> Arc<Self> {
        Arc::new(Self {
            island,
            settings,
            sound,
            rt,
            run: Mutex::new(Run {
                phase: PomodoroPhase::Idle,
                generation: 0,
                ends: Instant::now(),
            }),
        })
    }

    /// Mulai fokus; tanpa `duration` dipakai lama fokus dari setelan.
    pub fn start_focus(self: &Arc<Self>, duration: Option<Duration>) {
        let duration = duration
            .map(|d| d.clamp(MIN_FOCUS, MAX_FOCUS))
            .unwrap_or_else(|| self.minutes(|s| s.focus_minutes));
        self.begin(PomodoroPhase::Focus, duration);
    }

    pub fn stop(&self) {
        let mut run = self.run();
        run.phase = PomodoroPhase::Idle;
        run.generation += 1;
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

    /// Durasi dari setelan saat fase dimulai, jadi perubahan setelan berlaku di fase berikutnya.
    fn minutes(&self, pick: impl Fn(&Settings) -> u32) -> Duration {
        Duration::from_secs(u64::from(pick(&self.settings.get())) * 60)
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
        let settings = self.settings.get();
        let id = settings.language == Language::Id;
        if phase != PomodoroPhase::Idle && settings.sound_enabled {
            self.sound.chime();
        }
        match phase {
            PomodoroPhase::Focus => {
                self.island.dispatch(IslandAction::Notice(
                    if id {
                        "Fokus selesai, saatnya istirahat"
                    } else {
                        "Focus done, time for a break"
                    }
                    .into(),
                ));
                self.begin(PomodoroPhase::Break, self.minutes(|s| s.break_minutes));
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct Bell(AtomicUsize);
    impl SoundPlayer for Bell {
        fn chime(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn fx_with(
        settings: Settings,
    ) -> (
        Arc<PomodoroService>,
        Arc<IslandService>,
        Arc<Bell>,
        tokio::runtime::Runtime,
    ) {
        let (settings, island, rt) = services(settings);
        let bell = Arc::new(Bell::default());
        let svc = PomodoroService::new(island.clone(), settings, bell.clone(), rt.handle().clone());
        (svc, island, bell, rt)
    }

    fn fx() -> (
        Arc<PomodoroService>,
        Arc<IslandService>,
        Arc<Bell>,
        tokio::runtime::Runtime,
    ) {
        fx_with(Settings::default())
    }

    #[test]
    fn fokus_lalu_istirahat_lalu_selesai_dan_tidak_menyentuh_jangan_ganggu() {
        let (svc, island, bell, _rt) = fx();
        svc.start_focus(None);
        assert_eq!(svc.status().0, PomodoroPhase::Focus);
        assert!(
            !island.snapshot().dnd,
            "fokus tidak menyalakan Jangan Ganggu"
        );

        svc.finish(99); // timer usang
        assert_eq!(svc.status().0, PomodoroPhase::Focus);
        assert_eq!(bell.0.load(Ordering::SeqCst), 0);

        svc.finish(1);
        assert_eq!(svc.status().0, PomodoroPhase::Break);
        assert!(matches!(
            island.snapshot().view,
            View::Expanded(Content::Notice(_))
        ));
        assert_eq!(bell.0.load(Ordering::SeqCst), 1);

        svc.finish(2);
        assert_eq!(svc.status(), (PomodoroPhase::Idle, 0));
        assert_eq!(bell.0.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn jangan_ganggu_milik_pengguna_tidak_dimatikan_oleh_fokus() {
        let (svc, island, _bell, _rt) = fx();
        island.dispatch(IslandAction::DoNotDisturb(true));
        svc.start_focus(None);
        svc.finish(1);
        svc.stop();
        assert!(island.snapshot().dnd);
    }

    #[test]
    fn suara_mati_di_setelan_tidak_berbunyi() {
        let (svc, _island, bell, _rt) = fx_with(Settings {
            sound_enabled: false,
            ..Default::default()
        });
        svc.start_focus(None);
        svc.finish(1);
        assert_eq!(bell.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn durasi_custom_dijepit_dan_default_mengikuti_setelan() {
        let (svc, _island, _bell, _rt) = fx_with(Settings {
            focus_minutes: 50,
            break_minutes: 10,
            ..Default::default()
        });
        assert_eq!(svc.minutes(|s| s.focus_minutes), Duration::from_secs(3000));
        assert_eq!(svc.minutes(|s| s.break_minutes), Duration::from_secs(600));
        svc.start_focus(Some(Duration::from_secs(90)));
        let secs = svc.status().1;
        assert!((85..=90).contains(&secs), "{secs}");
        svc.start_focus(Some(Duration::from_secs(10 * 24 * 3600)));
        assert!(svc.status().1 <= 24 * 3600);
        svc.start_focus(Some(Duration::ZERO));
        assert!(svc.status().1 <= 1);
    }

    #[test]
    fn berhenti_di_tengah_fokus_membatalkan_timer_tanpa_bunyi() {
        let (svc, island, bell, _rt) = fx();
        svc.start_focus(None);
        svc.stop();
        assert_eq!(svc.status().0, PomodoroPhase::Idle);
        svc.finish(1);
        assert_eq!(svc.status().0, PomodoroPhase::Idle);
        assert_eq!(island.snapshot().view, View::Idle);
        assert_eq!(bell.0.load(Ordering::SeqCst), 0);
    }
}
