use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::domain::ports::NetCounters;

/// Kecepatan jaringan dalam byte per detik.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetRate {
    pub down: u32,
    pub up: u32,
}

/// Menghitung kecepatan dari selisih total byte antar dua pembacaan.
pub struct NetService {
    counters: Arc<dyn NetCounters>,
    last: Mutex<Option<(Instant, u64, u64)>>,
}

impl NetService {
    pub fn new(counters: Arc<dyn NetCounters>) -> Arc<Self> {
        Arc::new(Self {
            counters,
            last: Mutex::new(None),
        })
    }

    pub fn rate(&self) -> Option<NetRate> {
        self.rate_at(Instant::now())
    }

    /// Pembacaan pertama hanya menjadi acuan (nol); total yang turun (adaptor di-reset) dihitung nol.
    pub fn rate_at(&self, now: Instant) -> Option<NetRate> {
        let (down, up) = self.counters.totals()?;
        let prev = self
            .last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace((now, down, up));
        let zero = Some(NetRate { down: 0, up: 0 });
        let Some((t, d0, u0)) = prev else {
            return zero;
        };
        let secs = now.duration_since(t).as_secs_f64();
        if secs < 0.05 {
            return zero;
        }
        let per_sec = |new: u64, old: u64| {
            (new.saturating_sub(old) as f64 / secs).min(u32::MAX as f64) as u32
        };
        Some(NetRate {
            down: per_sec(down, d0),
            up: per_sec(up, u0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Fake(Mutex<Option<(u64, u64)>>);
    impl NetCounters for Fake {
        fn totals(&self) -> Option<(u64, u64)> {
            *self.0.lock().unwrap()
        }
    }

    #[test]
    fn kecepatan_dari_selisih_total_dan_tahan_reset() {
        let fake = Arc::new(Fake(Mutex::new(Some((1_000, 500)))));
        let svc = NetService::new(fake.clone());
        let t0 = Instant::now();
        assert_eq!(svc.rate_at(t0), Some(NetRate { down: 0, up: 0 }));
        *fake.0.lock().unwrap() = Some((1_000 + 52_000, 500 + 8_000));
        let r = svc.rate_at(t0 + Duration::from_secs(1)).unwrap();
        assert_eq!((r.down, r.up), (52_000, 8_000));
        // total turun (adaptor di-reset) -> nol, bukan angka raksasa
        *fake.0.lock().unwrap() = Some((10, 10));
        let r = svc.rate_at(t0 + Duration::from_secs(3)).unwrap();
        assert_eq!((r.down, r.up), (0, 0));
    }

    #[test]
    fn tanpa_dukungan_os_hasilnya_none() {
        let svc = NetService::new(Arc::new(Fake(Mutex::new(None))));
        assert_eq!(svc.rate(), None);
    }
}
