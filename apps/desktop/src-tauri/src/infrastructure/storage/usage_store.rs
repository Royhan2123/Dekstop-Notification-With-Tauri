use std::sync::Arc;

use crate::domain::error::UsageError;
use crate::domain::health::DayUsage;
use crate::domain::ports::UsageStore;

use super::sqlite::Database;

pub struct SqliteUsageStore(pub Arc<Database>);

fn err(e: rusqlite::Error) -> UsageError {
    UsageError::Storage(e.to_string())
}

impl UsageStore for SqliteUsageStore {
    fn record(
        &self,
        date: &str,
        minute: u32,
        active: bool,
        brightness: Option<u8>,
    ) -> Result<(), UsageError> {
        self.0
            .conn()
            .execute(
                "INSERT INTO usage_minutes (date, minute, active, brightness) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(date, minute) DO UPDATE SET active = excluded.active, brightness = excluded.brightness",
                rusqlite::params![date, minute, active, brightness],
            )
            .map(|_| ())
            .map_err(err)
    }

    fn days(&self, dates: &[String]) -> Result<Vec<DayUsage>, UsageError> {
        let conn = self.0.conn();
        let mut q = conn
            .prepare("SELECT COALESCE(SUM(active), 0), AVG(brightness) FROM usage_minutes WHERE date = ?1")
            .map_err(err)?;
        dates
            .iter()
            .map(|date| {
                q.query_row([date], |r| {
                    Ok(DayUsage {
                        date: date.clone(),
                        active_min: r.get(0)?,
                        avg_brightness: r.get::<_, Option<f64>>(1)?.map(|b| b.round() as u8),
                    })
                })
                .map_err(err)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catatan_per_menit_diperbarui_dan_dirangkum_per_hari() {
        let store = SqliteUsageStore(Arc::new(Database::open_in_memory().unwrap()));
        store.record("2026-11-02", 10, true, Some(60)).unwrap();
        store.record("2026-11-02", 11, true, Some(80)).unwrap();
        store.record("2026-11-02", 12, false, None).unwrap();
        store.record("2026-11-02", 12, true, None).unwrap(); // menit yang sama ditimpa
        let days = store
            .days(&["2026-11-01".to_string(), "2026-11-02".to_string()])
            .unwrap();
        assert_eq!(
            days[0],
            DayUsage {
                date: "2026-11-01".into(),
                active_min: 0,
                avg_brightness: None
            }
        );
        assert_eq!(days[1].active_min, 3);
        assert_eq!(days[1].avg_brightness, Some(70));
    }
}
