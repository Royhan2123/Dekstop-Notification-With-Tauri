use std::sync::Arc;

use crate::domain::error::StartupError;
use crate::domain::ports::StartupStore;
use crate::domain::startup::StartupApp;

use super::sqlite::Database;

pub struct SqliteStartupStore(pub Arc<Database>);

fn err(e: rusqlite::Error) -> StartupError {
    StartupError::Storage(e.to_string())
}

impl StartupStore for SqliteStartupStore {
    fn list(&self) -> Result<Vec<StartupApp>, StartupError> {
        let conn = self.0.conn();
        let mut q = conn
            .prepare("SELECT id, name, path, delay_secs FROM startup_apps ORDER BY position, id")
            .map_err(err)?;
        q.query_map([], |r| {
            Ok(StartupApp {
                id: r.get(0)?,
                name: r.get(1)?,
                path: r.get(2)?,
                delay_secs: r.get(3)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    fn add(&self, name: &str, path: &str, delay_secs: u32) -> Result<(), StartupError> {
        self.0
            .conn()
            .execute(
                "INSERT INTO startup_apps (name, path, delay_secs, position)
                 VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), 0) + 1 FROM startup_apps))",
                rusqlite::params![name, path, delay_secs],
            )
            .map(|_| ())
            .map_err(err)
    }

    fn set_delay(&self, id: i32, delay_secs: u32) -> Result<(), StartupError> {
        let n = self
            .0
            .conn()
            .execute(
                "UPDATE startup_apps SET delay_secs = ?2 WHERE id = ?1",
                rusqlite::params![id, delay_secs],
            )
            .map_err(err)?;
        if n == 0 {
            Err(StartupError::NotFound)
        } else {
            Ok(())
        }
    }

    fn reorder(&self, ids: &[i32]) -> Result<(), StartupError> {
        let mut conn = self.0.conn();
        let tx = conn.transaction().map_err(err)?;
        for (pos, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE startup_apps SET position = ?2 WHERE id = ?1",
                rusqlite::params![id, pos as i64],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)
    }

    fn remove(&self, id: i32) -> Result<(), StartupError> {
        self.0
            .conn()
            .execute("DELETE FROM startup_apps WHERE id = ?1", [id])
            .map(|_| ())
            .map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> SqliteStartupStore {
        SqliteStartupStore(Arc::new(Database::open_in_memory().unwrap()))
    }

    #[test]
    fn tambah_urut_ubah_jeda_geser_dan_hapus() {
        let s = store();
        s.add("a", "C:\\a.exe", 0).unwrap();
        s.add("b", "C:\\b.exe", 5).unwrap();
        s.add("c", "C:\\c.exe", 0).unwrap();
        let ids: Vec<i32> = s.list().unwrap().iter().map(|a| a.id).collect();
        s.reorder(&[ids[2], ids[0], ids[1]]).unwrap();
        let names: Vec<_> = s.list().unwrap().into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["c", "a", "b"]);
        s.set_delay(ids[0], 12).unwrap();
        assert_eq!(s.list().unwrap()[1].delay_secs, 12);
        assert!(matches!(s.set_delay(999, 1), Err(StartupError::NotFound)));
        s.remove(ids[1]).unwrap();
        assert_eq!(s.list().unwrap().len(), 2);
    }
}
