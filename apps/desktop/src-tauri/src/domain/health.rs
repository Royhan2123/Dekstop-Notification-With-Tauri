//! Aturan kesehatan layar. Murni: snapshot masuk, daftar peringatan keluar.
use crate::domain::model::settings::{HealthRules, Language};

/// Layar dianggap aktif bila input terakhir kurang dari ini.
pub const IDLE_LIMIT_SECS: u64 = 180;
/// Istirahat selama ini mengulang hitungan pemakaian nonstop.
pub const BREAK_MINUTES: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HealthKind {
    LateNight,
    ContinuousUse,
    BrightAtNight,
    TwentyTwentyTwenty,
}

impl HealthKind {
    /// Jeda minimum (menit) sebelum peringatan yang sama muncul lagi.
    pub fn cooldown_minutes(self) -> u64 {
        match self {
            Self::ContinuousUse => 30,
            Self::TwentyTwentyTwenty => 20,
            Self::BrightAtNight | Self::LateNight => 60,
        }
    }
}

/// Ringkasan satu hari untuk grafik.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayUsage {
    pub date: String,
    pub active_min: u32,
    pub avg_brightness: Option<u8>,
}

pub struct HealthSnapshot {
    /// Jam lokal 0-23.
    pub hour: u32,
    /// Menit aktif berturut-turut sejak istirahat terakhir.
    pub continuous_min: u32,
    pub brightness: Option<u8>,
}

fn is_night(hour: u32) -> bool {
    !(5..19).contains(&hour)
}

/// Aturan yang terpenuhi, terpenting dulu; pengulangan diatur layanan lewat `cooldown_minutes`.
pub fn evaluate(rules: &HealthRules, s: &HealthSnapshot) -> Vec<HealthKind> {
    let mut out = Vec::new();
    if rules.late_night && s.continuous_min > 0 && (s.hour >= 22 || s.hour < 4) {
        out.push(HealthKind::LateNight);
    }
    if rules.continuous && s.continuous_min >= 120 {
        out.push(HealthKind::ContinuousUse);
    }
    if rules.bright_night && is_night(s.hour) && s.brightness.is_some_and(|b| b >= 70) {
        out.push(HealthKind::BrightAtNight);
    }
    if rules.twenty_twenty_twenty && s.continuous_min >= 20 {
        out.push(HealthKind::TwentyTwentyTwenty);
    }
    out
}

pub fn message(kind: HealthKind, lang: Language) -> &'static str {
    match (kind, lang) {
        (HealthKind::LateNight, Language::Id) => {
            "Sudah lewat jam 22.00. Saatnya berhenti dan istirahat."
        }
        (HealthKind::LateNight, Language::En) => "It's past 10 PM. Time to stop and rest.",
        (HealthKind::ContinuousUse, Language::Id) => {
            "Sudah 2 jam lebih nonstop. Istirahat dulu ya."
        }
        (HealthKind::ContinuousUse, Language::En) => "Over 2 hours nonstop. Take a break.",
        (HealthKind::BrightAtNight, Language::Id) => {
            "Layar terlalu terang untuk malam hari. Turunkan kecerahan."
        }
        (HealthKind::BrightAtNight, Language::En) => {
            "Your screen is too bright for night. Lower the brightness."
        }
        (HealthKind::TwentyTwentyTwenty, Language::Id) => {
            "Lihat jauh 20 detik ya (aturan 20-20-20)."
        }
        (HealthKind::TwentyTwentyTwenty, Language::En) => {
            "Look far away for 20 seconds (20-20-20 rule)."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> HealthRules {
        HealthRules {
            twenty_twenty_twenty: true,
            ..Default::default()
        }
    }

    fn snap(hour: u32, continuous_min: u32, brightness: Option<u8>) -> HealthSnapshot {
        HealthSnapshot {
            hour,
            continuous_min,
            brightness,
        }
    }

    #[test]
    fn pemakaian_nonstop_mulai_dua_jam() {
        assert!(
            evaluate(&all(), &snap(14, 119, None))
                .iter()
                .all(|k| *k != HealthKind::ContinuousUse)
        );
        assert!(evaluate(&all(), &snap(14, 120, None)).contains(&HealthKind::ContinuousUse));
    }

    #[test]
    fn dua_puluh_dua_puluh_dua_puluh_mulai_menit_ke_dua_puluh() {
        assert!(evaluate(&all(), &snap(14, 19, None)).is_empty());
        assert_eq!(
            evaluate(&all(), &snap(14, 20, None)),
            vec![HealthKind::TwentyTwentyTwenty]
        );
    }

    #[test]
    fn terang_di_malam_hanya_bila_kecerahan_diketahui_dan_tinggi() {
        assert_eq!(
            evaluate(&all(), &snap(20, 0, Some(70))),
            vec![HealthKind::BrightAtNight]
        );
        assert!(evaluate(&all(), &snap(20, 0, Some(69))).is_empty());
        assert!(evaluate(&all(), &snap(20, 0, None)).is_empty());
        assert!(evaluate(&all(), &snap(13, 0, Some(100))).is_empty());
    }

    #[test]
    fn lembur_setelah_jam_22_sampai_subuh_hanya_saat_aktif() {
        assert!(evaluate(&all(), &snap(22, 1, None)).contains(&HealthKind::LateNight));
        assert!(evaluate(&all(), &snap(3, 1, None)).contains(&HealthKind::LateNight));
        assert!(evaluate(&all(), &snap(21, 1, None)).is_empty());
        assert!(evaluate(&all(), &snap(23, 0, None)).is_empty());
    }

    #[test]
    fn aturan_yang_dimatikan_tidak_pernah_menyala_dan_urutan_berdasar_kepentingan() {
        let off = HealthRules {
            continuous: false,
            bright_night: false,
            late_night: false,
            twenty_twenty_twenty: false,
        };
        assert!(evaluate(&off, &snap(23, 300, Some(100))).is_empty());
        assert_eq!(
            evaluate(&all(), &snap(23, 300, Some(100))),
            vec![
                HealthKind::LateNight,
                HealthKind::ContinuousUse,
                HealthKind::BrightAtNight,
                HealthKind::TwentyTwentyTwenty
            ]
        );
    }

    #[test]
    fn pesan_tersedia_dalam_dua_bahasa() {
        for k in [
            HealthKind::LateNight,
            HealthKind::ContinuousUse,
            HealthKind::BrightAtNight,
            HealthKind::TwentyTwentyTwenty,
        ] {
            assert_ne!(message(k, Language::Id), message(k, Language::En));
        }
    }
}
