//! Rapat berikutnya dari berkas kalender iCal (RFC 5545): hanya bagian yang dibutuhkan pulau.
//!
//! shortcut: zona waktu `TZID=` dianggap sama dengan zona komputer, dan pengulangan hanya
//! `DAILY` dan `WEEKLY` (dengan `INTERVAL`, `BYDAY`, `COUNT`, `UNTIL`, `EXDATE`); rapat bulanan/tahunan
//! hanya terbaca kemunculan pertamanya. Upgrade bila ada kalender yang meleset karena itu.
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

/// Seberapa jauh ke depan rapat dicari.
const HORIZON_DAYS: i64 = 14;
/// Batas langkah penelusuran satu seri berulang (melindungi dari kalender rusak).
const MAX_STEPS: u32 = 5_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    pub title: String,
    /// Waktu mulai menurut jam dinding komputer.
    pub start: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Freq {
    Daily,
    Weekly,
}

#[derive(Clone, Debug)]
struct Rule {
    freq: Freq,
    interval: u32,
    by_day: Vec<Weekday>,
    until: Option<NaiveDateTime>,
    count: Option<u32>,
}

#[derive(Default)]
struct Event {
    uid: String,
    title: String,
    start: Option<NaiveDateTime>,
    all_day: bool,
    cancelled: bool,
    rule: Option<Rule>,
    exdates: Vec<NaiveDateTime>,
    recurrence_id: Option<NaiveDateTime>,
}

/// Sambung baris lanjutan (diawali spasi/tab) ke baris sebelumnya.
fn unfold(ics: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in ics.lines() {
        match (raw.strip_prefix([' ', '\t']), lines.last_mut()) {
            (Some(rest), Some(last)) => last.push_str(rest),
            _ => lines.push(raw.to_string()),
        }
    }
    lines
}

fn unescape(s: &str) -> String {
    s.replace("\\n", " ")
        .replace("\\N", " ")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

/// `YYYYMMDD` (seharian) atau `YYYYMMDDTHHMMSS[Z]`; waktu UTC (`Z`) digeser ke zona komputer.
fn parse_time(value: &str, offset_secs: i32) -> Option<(NaiveDateTime, bool)> {
    let value = value.trim();
    if value.len() == 8 {
        let date = NaiveDate::parse_from_str(value, "%Y%m%d").ok()?;
        return Some((date.and_time(NaiveTime::MIN), true));
    }
    let utc = value.ends_with('Z');
    let body = value.trim_end_matches('Z');
    let t = NaiveDateTime::parse_from_str(body, "%Y%m%dT%H%M%S").ok()?;
    let t = if utc {
        t + Duration::seconds(i64::from(offset_secs))
    } else {
        t
    };
    Some((t, false))
}

fn weekday(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

/// `None` untuk frekuensi yang tidak didukung (acara dianggap tidak berulang).
fn parse_rule(value: &str, offset_secs: i32) -> Option<Rule> {
    let mut rule = Rule {
        freq: Freq::Daily,
        interval: 1,
        by_day: Vec::new(),
        until: None,
        count: None,
    };
    let mut freq = None;
    for part in value.split(';') {
        let (k, v) = part.split_once('=')?;
        match k {
            "FREQ" => {
                freq = match v {
                    "DAILY" => Some(Freq::Daily),
                    "WEEKLY" => Some(Freq::Weekly),
                    _ => None,
                }
            }
            "INTERVAL" => rule.interval = v.parse().ok().filter(|n| *n >= 1).unwrap_or(1),
            "COUNT" => rule.count = v.parse().ok(),
            "UNTIL" => {
                rule.until = parse_time(v, offset_secs).map(|(t, all_day)| {
                    // UNTIL berupa tanggal berarti sampai akhir hari itu.
                    if all_day { t + Duration::days(1) } else { t }
                })
            }
            "BYDAY" => {
                // "2MO" (hari ke-n dalam bulan) hanya berarti untuk pengulangan bulanan; abaikan angkanya.
                rule.by_day = v
                    .split(',')
                    .filter_map(|d| {
                        weekday(d.trim_start_matches(|c: char| {
                            c.is_ascii_digit() || c == '-' || c == '+'
                        }))
                    })
                    .collect();
            }
            _ => {}
        }
    }
    rule.freq = freq?;
    Some(rule)
}

fn parse_events(ics: &str, offset_secs: i32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut current: Option<Event> = None;
    for line in unfold(ics) {
        if line == "BEGIN:VEVENT" {
            current = Some(Event::default());
            continue;
        }
        if line == "END:VEVENT" {
            events.extend(current.take());
            continue;
        }
        let Some(ev) = current.as_mut() else { continue };
        let Some((head, value)) = line.split_once(':') else {
            continue;
        };
        let name = head
            .split(';')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        match name.as_str() {
            "UID" => ev.uid = value.trim().to_string(),
            "SUMMARY" => ev.title = unescape(value).trim().to_string(),
            "STATUS" => ev.cancelled = value.trim().eq_ignore_ascii_case("CANCELLED"),
            "DTSTART" => {
                if let Some((t, all_day)) = parse_time(value, offset_secs) {
                    ev.start = Some(t);
                    ev.all_day = all_day;
                }
            }
            "RRULE" => ev.rule = parse_rule(value, offset_secs),
            "EXDATE" => ev.exdates.extend(
                value
                    .split(',')
                    .filter_map(|v| parse_time(v, offset_secs).map(|(t, _)| t)),
            ),
            "RECURRENCE-ID" => ev.recurrence_id = parse_time(value, offset_secs).map(|(t, _)| t),
            _ => {}
        }
    }
    events
}

/// Kemunculan sebuah acara dari DTSTART sampai `horizon` (termasuk yang sudah lewat, agar COUNT benar).
fn occurrences(ev: &Event, start: NaiveDateTime, horizon: NaiveDateTime) -> Vec<NaiveDateTime> {
    let Some(rule) = &ev.rule else {
        return vec![start];
    };
    let mut out = Vec::new();
    let emit = |t: NaiveDateTime, out: &mut Vec<NaiveDateTime>| -> bool {
        if t > horizon || rule.until.is_some_and(|u| t > u) {
            return false;
        }
        out.push(t);
        rule.count.is_none_or(|c| (out.len() as u32) < c)
    };
    match rule.freq {
        Freq::Daily => {
            for k in 0..MAX_STEPS {
                let t = start + Duration::days(i64::from(k) * i64::from(rule.interval));
                if !emit(t, &mut out) {
                    break;
                }
            }
        }
        Freq::Weekly => {
            let days = if rule.by_day.is_empty() {
                vec![start.weekday()]
            } else {
                rule.by_day.clone()
            };
            let monday =
                start.date() - Duration::days(i64::from(start.weekday().num_days_from_monday()));
            'weeks: for w in 0..MAX_STEPS {
                let week = monday + Duration::days(i64::from(w) * 7 * i64::from(rule.interval));
                let mut in_week: Vec<NaiveDateTime> = days
                    .iter()
                    .map(|d| {
                        (week + Duration::days(i64::from(d.num_days_from_monday())))
                            .and_time(start.time())
                    })
                    .filter(|t| *t >= start)
                    .collect();
                in_week.sort();
                for t in in_week {
                    if !emit(t, &mut out) {
                        break 'weeks;
                    }
                }
                if week.and_time(start.time()) > horizon {
                    break;
                }
            }
        }
    }
    out
}

/// Rapat terdekat yang dimulai pada atau setelah `now` (jam dinding komputer), dalam dua minggu ke depan.
/// `offset_secs` = selisih zona komputer dari UTC, untuk waktu berakhiran `Z`.
pub fn next_meeting(ics: &str, now: NaiveDateTime, offset_secs: i32) -> Option<Meeting> {
    let horizon = now + Duration::days(HORIZON_DAYS);
    let events = parse_events(ics, offset_secs);
    events
        .iter()
        .filter(|e| !e.cancelled && !e.all_day)
        .filter_map(|e| e.start.map(|s| (e, s)))
        .flat_map(|(e, start)| {
            // Acara hasil penggantian satu kemunculan (RECURRENCE-ID) menggantikan yang asli.
            let replaced: Vec<NaiveDateTime> = events
                .iter()
                .filter(|o| {
                    o.uid == e.uid
                        && !e.uid.is_empty()
                        && o.recurrence_id.is_some()
                        && o.rule.is_none()
                })
                .filter_map(|o| o.recurrence_id)
                .collect();
            let is_series = e.rule.is_some() && e.recurrence_id.is_none();
            occurrences(e, start, horizon)
                .into_iter()
                .filter(move |t| !is_series || (!e.exdates.contains(t) && !replaced.contains(t)))
                .filter(|t| *t >= now)
                .map(|t| Meeting {
                    title: if e.title.is_empty() {
                        "(tanpa judul)".into()
                    } else {
                        e.title.clone()
                    },
                    start: t,
                })
                .collect::<Vec<_>>()
        })
        .min_by_key(|m| m.start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    fn cal(events: &[&str]) -> String {
        let body: String = events
            .iter()
            .map(|e| format!("BEGIN:VEVENT\n{e}\nEND:VEVENT\n"))
            .collect();
        format!("BEGIN:VCALENDAR\nVERSION:2.0\n{body}END:VCALENDAR\n")
    }

    #[test]
    fn rapat_terdekat_dari_beberapa_acara_dan_waktu_utc_digeser_ke_zona_komputer() {
        let ics = cal(&[
            "UID:a\nSUMMARY:Nanti sore\nDTSTART:20261103T080000Z",
            "UID:b\nSUMMARY:Sebentar lagi\nDTSTART;TZID=Asia/Jakarta:20261103T143000",
            "UID:c\nSUMMARY:Sudah lewat\nDTSTART:20261103T010000Z",
        ]);
        // sekarang 14:00 di zona UTC+7: acara A = 15:00 lokal, B = 14:30, C = 08:00 (lewat)
        let m = next_meeting(&ics, at("2026-11-03 14:00"), 7 * 3600).unwrap();
        assert_eq!(m.title, "Sebentar lagi");
        assert_eq!(m.start, at("2026-11-03 14:30"));
        let m = next_meeting(&ics, at("2026-11-03 14:45"), 7 * 3600).unwrap();
        assert_eq!(m.title, "Nanti sore");
        assert_eq!(m.start, at("2026-11-03 15:00"));
        assert!(next_meeting(&ics, at("2026-11-03 16:00"), 7 * 3600).is_none());
    }

    #[test]
    fn acara_seharian_dibatalkan_dan_kosong_diabaikan() {
        let ics = cal(&[
            "UID:a\nSUMMARY:Libur\nDTSTART;VALUE=DATE:20261104",
            "UID:b\nSUMMARY:Batal\nSTATUS:CANCELLED\nDTSTART:20261104T020000Z",
        ]);
        assert!(next_meeting(&ics, at("2026-11-03 09:00"), 0).is_none());
        assert!(next_meeting("", at("2026-11-03 09:00"), 0).is_none());
        assert!(next_meeting("sampah bukan kalender", at("2026-11-03 09:00"), 0).is_none());
    }

    #[test]
    fn baris_lanjutan_dan_karakter_escape_dibaca() {
        let ics = "BEGIN:VEVENT\nUID:a\nSUMMARY:Rapat\\, tim\n  produk\nDTSTART:20261103T100000\nEND:VEVENT\n";
        let m = next_meeting(ics, at("2026-11-03 09:00"), 0).unwrap();
        assert_eq!(m.title, "Rapat, tim produk");
    }

    #[test]
    fn berulang_harian_dengan_interval_count_dan_until() {
        let daily =
            cal(&["UID:a\nSUMMARY:Standup\nDTSTART:20261101T090000\nRRULE:FREQ=DAILY;INTERVAL=2"]);
        // 1, 3, 5 Nov ...: setelah 3 Nov 09:00 berikutnya 5 Nov
        assert_eq!(
            next_meeting(&daily, at("2026-11-03 09:01"), 0)
                .unwrap()
                .start,
            at("2026-11-05 09:00")
        );
        let counted =
            cal(&["UID:a\nSUMMARY:Sprint\nDTSTART:20261101T090000\nRRULE:FREQ=DAILY;COUNT=2"]);
        assert!(
            next_meeting(&counted, at("2026-11-03 00:00"), 0).is_none(),
            "hanya 1 dan 2 Nov"
        );
        let until = cal(&[
            "UID:a\nSUMMARY:Sampai\nDTSTART:20261101T090000\nRRULE:FREQ=DAILY;UNTIL=20261104T090000",
        ]);
        assert_eq!(
            next_meeting(&until, at("2026-11-03 09:01"), 0)
                .unwrap()
                .start,
            at("2026-11-04 09:00")
        );
        assert!(next_meeting(&until, at("2026-11-04 09:01"), 0).is_none());
    }

    #[test]
    fn berulang_mingguan_dengan_byday_interval_dan_exdate() {
        // Selasa 3 Nov 2026; Senin dan Rabu tiap 2 minggu mulai Senin 2 Nov
        let weekly = cal(&[
            "UID:a\nSUMMARY:Review\nDTSTART:20261102T100000\nRRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE",
        ]);
        let m = next_meeting(&weekly, at("2026-11-03 12:00"), 0).unwrap();
        assert_eq!(m.start, at("2026-11-04 10:00"), "Rabu minggu yang sama");
        let m = next_meeting(&weekly, at("2026-11-04 10:01"), 0).unwrap();
        assert_eq!(m.start, at("2026-11-16 10:00"), "lompat dua minggu");
        let skipped = cal(&[
            "UID:a\nSUMMARY:Review\nDTSTART:20261102T100000\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE\nEXDATE:20261104T100000",
        ]);
        assert_eq!(
            next_meeting(&skipped, at("2026-11-03 12:00"), 0)
                .unwrap()
                .start,
            at("2026-11-09 10:00")
        );
        let plain = cal(&["UID:a\nSUMMARY:Mingguan\nDTSTART:20261027T150000\nRRULE:FREQ=WEEKLY"]);
        assert_eq!(
            next_meeting(&plain, at("2026-11-03 12:00"), 0)
                .unwrap()
                .start,
            at("2026-11-03 15:00")
        );
    }

    #[test]
    fn satu_kemunculan_yang_dipindah_menggantikan_yang_asli() {
        let ics = cal(&[
            "UID:s\nSUMMARY:Mingguan\nDTSTART:20261027T150000\nRRULE:FREQ=WEEKLY",
            "UID:s\nSUMMARY:Mingguan (pindah)\nRECURRENCE-ID:20261103T150000\nDTSTART:20261103T170000",
        ]);
        let m = next_meeting(&ics, at("2026-11-03 12:00"), 0).unwrap();
        assert_eq!(
            (m.title.as_str(), m.start),
            ("Mingguan (pindah)", at("2026-11-03 17:00"))
        );
        // jam 15:00 sudah digantikan; setelah jam 17:00 berikutnya minggu depan
        let m = next_meeting(&ics, at("2026-11-03 17:01"), 0).unwrap();
        assert_eq!(m.start, at("2026-11-10 15:00"));
    }

    #[test]
    fn frekuensi_bulanan_tidak_didukung_hanya_kemunculan_pertama() {
        let ics = cal(&["UID:a\nSUMMARY:Bulanan\nDTSTART:20261001T090000\nRRULE:FREQ=MONTHLY"]);
        assert!(next_meeting(&ics, at("2026-11-03 09:00"), 0).is_none());
    }
}
