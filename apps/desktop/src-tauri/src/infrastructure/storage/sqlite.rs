use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

type DbResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// SQLite lokal (`isla.db`) dengan migrasi berversi. Tabel baru = migrasi baru di `migrations()`.
pub struct Database(Mutex<Connection>);

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE usage_minutes (
            date TEXT NOT NULL,
            minute INTEGER NOT NULL,
            active INTEGER NOT NULL,
            brightness INTEGER,
            PRIMARY KEY (date, minute)
        );",
        ),
        M::up(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0
            );",
        ),
        M::up(
            "CREATE TABLE chat_turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question TEXT NOT NULL,
                answer TEXT NOT NULL,
                model TEXT NOT NULL
            );",
        ),
        M::up(
            "ALTER TABLE tasks ADD COLUMN note INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE tasks ADD COLUMN body TEXT NOT NULL DEFAULT '';",
        ),
    ])
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> DbResult<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> DbResult<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        migrations().to_latest(&mut conn)?;
        Ok(Self(Mutex::new(conn)))
    }

    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrasi_valid_dan_membuat_tabel_pemakaian() {
        assert!(migrations().validate().is_ok());
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute(
                "INSERT INTO usage_minutes VALUES ('2026-11-02', 10, 1, 70)",
                [],
            )
            .unwrap();
        let n: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM usage_minutes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn membuka_file_dua_kali_tidak_mengulang_migrasi() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data").join("isla.db");
        drop(Database::open(&path).unwrap());
        assert!(Database::open(&path).is_ok());
    }
}
