use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::error::UsageError;
use crate::domain::health::{
    BREAK_MINUTES, DayUsage, HealthKind, HealthSnapshot, IDLE_LIMIT_SECS, evaluate, message,
};
use crate::domain::island::action::IslandAction;
use crate::domain::model::HealthAlert;
use crate::domain::model::settings::Language;
use crate::domain::ports::{BrightnessReader, IdleDetector, UsageStore};

#[derive(Default)]
struct Tracker {
    continuous_min: u32,
    idle_streak: u32,
    ticks: u64,
    last_fired: HashMap<HealthKind, u64>,
    brightness: Option<u8>,
}

/// Ringkasan untuk kartu "hari ini" dan grafik mingguan.
pub struct HealthSummary {
    pub today_active_min: u32,
    pub brightness: Option<u8>,
    /// Tujuh hari terakhir, terlama dulu; yang terakhir adalah hari ini.
    pub days: Vec<DayUsage>,
}

/// Mencatat pemakaian tiap menit dan memunculkan pengingat. `tick` dipanggil ticker 1 menit.
pub struct HealthService {
    idle: Arc<dyn IdleDetector>,
    brightness: Arc<dyn BrightnessReader>,
    store: Arc<dyn UsageStore>,
    settings: Arc<SettingsService>,
    island: Arc<IslandService>,
    tracker: Mutex<Tracker>,
}

fn date_key(t: NaiveDateTime) -> String {
    format!("{:04}-{:02}-{:02}", t.year(), t.month(), t.day())
}

fn duration_text(minutes: u32, lang: Language) -> String {
    let (h, m) = (minutes / 60, minutes % 60);
    match lang {
        Language::Id => format!("{h}j {m}m"),
        Language::En => format!("{h}h {m}m"),
    }
}

impl HealthService {
    pub fn new(
        idle: Arc<dyn IdleDetector>,
        brightness: Arc<dyn BrightnessReader>,
        store: Arc<dyn UsageStore>,
        settings: Arc<SettingsService>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            idle,
            brightness,
            store,
            settings,
            island,
            tracker: Mutex::new(Tracker::default()),
        })
    }

    /// Satu menit berlalu. Memblokir (membaca kecerahan, SQLite): panggil dari thread non-UI.
    pub fn tick(&self, now: NaiveDateTime) {
        let idle_secs = self.idle.idle_seconds().unwrap_or(0);
        let active = idle_secs < IDLE_LIMIT_SECS;
        let brightness = self.brightness.read();
        if let Err(e) = self.store.record(
            &date_key(now),
            now.hour() * 60 + now.minute(),
            active,
            brightness,
        ) {
            tracing::warn!("{e}");
        }

        let settings = self.settings.get();
        let (kind, lang) = {
            let mut t = self.tracker();
            t.ticks += 1;
            t.brightness = brightness;
            if active {
                t.idle_streak = 0;
                t.continuous_min += 1;
            } else {
                t.idle_streak += 1;
                if t.idle_streak >= BREAK_MINUTES {
                    t.continuous_min = 0;
                }
            }
            if !settings.health_enabled || !active {
                return;
            }
            let snapshot = HealthSnapshot {
                hour: now.hour(),
                continuous_min: t.continuous_min,
                brightness,
            };
            let ticks = t.ticks;
            let due = evaluate(&settings.health_rules, &snapshot)
                .into_iter()
                .find(|k| {
                    t.last_fired
                        .get(k)
                        .is_none_or(|last| ticks - last >= k.cooldown_minutes())
                });
            if let Some(k) = due {
                t.last_fired.insert(k, ticks);
            }
            (due, settings.language)
        };
        if let Some(kind) = kind {
            self.island.dispatch(IslandAction::HealthAlert(HealthAlert {
                message: message(kind, lang).into(),
            }));
        }
    }

    pub fn summary(&self, now: NaiveDateTime) -> Result<HealthSummary, UsageError> {
        let dates: Vec<String> = (0..7)
            .rev()
            .map(|back| date_key(now - Duration::days(back)))
            .collect();
        let days = self.store.days(&dates)?;
        Ok(HealthSummary {
            today_active_min: days.last().map_or(0, |d| d.active_min),
            brightness: self.tracker().brightness,
            days,
        })
    }

    /// Klik pulau saat idle: tampilkan ringkasan hari ini sebagai info singkat.
    pub fn peek(&self, now: NaiveDateTime) -> Result<(), UsageError> {
        let lang = self.settings.get().language;
        let s = self.summary(now)?;
        let (label, bright) = match lang {
            Language::Id => ("Hari ini aktif", "kecerahan"),
            Language::En => ("Active today", "brightness"),
        };
        let mut text = format!("{label} {}", duration_text(s.today_active_min, lang));
        if let Some(b) = s.brightness {
            text.push_str(&format!(" · {bright} {b}%"));
        }
        self.island.dispatch(IslandAction::UserNotice(text));
        Ok(())
    }

    fn tracker(&self) -> MutexGuard<'_, Tracker> {
        self.tracker.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::settings::{HealthRules, Settings};
    use chrono::NaiveDate;

    struct Idle(Mutex<Option<u64>>);
    impl IdleDetector for Idle {
        fn idle_seconds(&self) -> Option<u64> {
            *self.0.lock().unwrap()
        }
    }
    struct Bright(Option<u8>);
    impl BrightnessReader for Bright {
        fn read(&self) -> Option<u8> {
            self.0
        }
    }
    type Row = (String, u32, bool, Option<u8>);
    #[derive(Default)]
    struct Store(Mutex<Vec<Row>>);
    impl UsageStore for Store {
        fn record(&self, d: &str, m: u32, a: bool, b: Option<u8>) -> Result<(), UsageError> {
            self.0.lock().unwrap().push((d.into(), m, a, b));
            Ok(())
        }
        fn days(&self, dates: &[String]) -> Result<Vec<DayUsage>, UsageError> {
            let rows = self.0.lock().unwrap();
            Ok(dates
                .iter()
                .map(|d| DayUsage {
                    date: d.clone(),
                    active_min: rows.iter().filter(|r| &r.0 == d && r.2).count() as u32,
                    avg_brightness: None,
                })
                .collect())
        }
    }

    struct Fx {
        svc: Arc<HealthService>,
        island: Arc<IslandService>,
        idle: Arc<Idle>,
        store: Arc<Store>,
        _rt: tokio::runtime::Runtime,
    }

    fn fixture(settings: Settings, brightness: Option<u8>) -> Fx {
        let (sv, island, rt) = services(settings);
        let idle = Arc::new(Idle(Mutex::new(Some(0))));
        let store = Arc::new(Store::default());
        let svc = HealthService::new(
            idle.clone(),
            Arc::new(Bright(brightness)),
            store.clone(),
            sv,
            island.clone(),
        );
        Fx {
            svc,
            island,
            idle,
            store,
            _rt: rt,
        }
    }

    fn at(hour: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 2)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap()
    }

    fn alert(f: &Fx) -> Option<String> {
        match f.island.snapshot().view {
            View::Expanded(Content::HealthAlert(a)) => Some(a.message),
            _ => None,
        }
    }

    #[test]
    fn dua_jam_nonstop_memunculkan_saran_istirahat_dan_tidak_mengulang_sebelum_jeda() {
        let f = fixture(Settings::default(), None);
        for m in 0..119 {
            f.svc.tick(at(14, 0) + Duration::minutes(m));
        }
        assert_eq!(alert(&f), None);
        f.svc.tick(at(16, 0));
        assert!(alert(&f).unwrap().contains("2 jam"));
        f.island.dispatch(IslandAction::Dismissed);
        f.svc.tick(at(16, 1));
        assert_eq!(alert(&f), None, "masih dalam jeda 30 menit");
        for m in 2..=30 {
            f.svc.tick(at(16, 0) + Duration::minutes(m));
        }
        assert!(alert(&f).is_some(), "setelah 30 menit diingatkan lagi");
    }

    #[test]
    fn istirahat_lima_menit_mengulang_hitungan_dan_menit_idle_tercatat_tidak_aktif() {
        let f = fixture(Settings::default(), None);
        for m in 0..100 {
            f.svc.tick(at(10, 0) + Duration::minutes(m));
        }
        *f.idle.0.lock().unwrap() = Some(IDLE_LIMIT_SECS + 1);
        for m in 100..105 {
            f.svc.tick(at(10, 0) + Duration::minutes(m));
        }
        *f.idle.0.lock().unwrap() = Some(0);
        for m in 105..215 {
            f.svc.tick(at(10, 0) + Duration::minutes(m));
        }
        assert_eq!(alert(&f), None, "110 menit sejak istirahat: belum 2 jam");
        let rows = f.store.0.lock().unwrap();
        assert!(rows.iter().filter(|r| !r.2).count() == 5);
    }

    #[test]
    fn terang_di_malam_dan_lembur_mengikuti_aturan_serta_kunci_utama() {
        let f = fixture(Settings::default(), Some(90));
        f.svc.tick(at(23, 0));
        assert!(
            alert(&f).unwrap().contains("22.00"),
            "lembur lebih penting daripada terang"
        );

        let off = Settings {
            health_enabled: false,
            ..Default::default()
        };
        let f = fixture(off, Some(90));
        f.svc.tick(at(23, 0));
        assert_eq!(alert(&f), None);
        assert_eq!(
            f.store.0.lock().unwrap().len(),
            1,
            "pemakaian tetap dicatat"
        );

        let rules = HealthRules {
            late_night: false,
            ..Default::default()
        };
        let f = fixture(
            Settings {
                health_rules: rules,
                ..Default::default()
            },
            Some(90),
        );
        f.svc.tick(at(23, 0));
        assert!(alert(&f).unwrap().contains("terang"));
    }

    #[test]
    fn pengguna_idle_tidak_diganggu_dan_ringkasan_tujuh_hari() {
        let f = fixture(Settings::default(), Some(80));
        *f.idle.0.lock().unwrap() = Some(IDLE_LIMIT_SECS);
        f.svc.tick(at(23, 0));
        assert_eq!(alert(&f), None);
        *f.idle.0.lock().unwrap() = Some(0);
        f.svc.tick(at(8, 0));
        let s = f.svc.summary(at(9, 0)).unwrap();
        assert_eq!(
            (s.days.len(), s.today_active_min, s.brightness),
            (7, 1, Some(80))
        );
        assert_eq!(s.days[0].date, "2026-10-27");
        f.svc.peek(at(9, 0)).unwrap();
        match f.island.snapshot().view.content() {
            Some(Content::Notice(t)) => assert!(t.contains("0j 1m") && t.contains("80%"), "{t}"),
            other => panic!("{other:?}"),
        }
    }
}
