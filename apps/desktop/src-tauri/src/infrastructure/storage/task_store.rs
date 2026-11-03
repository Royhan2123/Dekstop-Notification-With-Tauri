use std::sync::Arc;

use crate::domain::error::TaskError;
use crate::domain::ports::TaskStore;
use crate::domain::task::Task;

use super::sqlite::Database;

pub struct SqliteTaskStore(pub Arc<Database>);

fn err(e: rusqlite::Error) -> TaskError {
    TaskError::Storage(e.to_string())
}

impl TaskStore for SqliteTaskStore {
    /// Yang belum selesai di atas, lalu yang selesai; di dalam kelompok, yang terbaru di bawah.
    fn list(&self) -> Result<Vec<Task>, TaskError> {
        let conn = self.0.conn();
        let mut q = conn
            .prepare("SELECT id, title, done FROM tasks ORDER BY done, id")
            .map_err(err)?;
        q.query_map([], |r| {
            Ok(Task {
                id: r.get(0)?,
                title: r.get(1)?,
                done: r.get(2)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    fn add(&self, title: &str) -> Result<(), TaskError> {
        self.0
            .conn()
            .execute("INSERT INTO tasks (title, done) VALUES (?1, 0)", [title])
            .map(|_| ())
            .map_err(err)
    }

    fn set_done(&self, id: i32, done: bool) -> Result<(), TaskError> {
        self.0
            .conn()
            .execute(
                "UPDATE tasks SET done = ?2 WHERE id = ?1",
                rusqlite::params![id, done],
            )
            .map(|_| ())
            .map_err(err)
    }

    fn remove(&self, id: i32) -> Result<(), TaskError> {
        self.0
            .conn()
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .map(|_| ())
            .map_err(err)
    }

    fn clear_done(&self) -> Result<(), TaskError> {
        self.0
            .conn()
            .execute("DELETE FROM tasks WHERE done = 1", [])
            .map(|_| ())
            .map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tambah_centang_urutan_dan_hapus() {
        let s = SqliteTaskStore(Arc::new(Database::open_in_memory().unwrap()));
        s.add("a").unwrap();
        s.add("b").unwrap();
        s.add("c").unwrap();
        let id_a = s.list().unwrap()[0].id;
        s.set_done(id_a, true).unwrap();
        let rows: Vec<_> = s
            .list()
            .unwrap()
            .into_iter()
            .map(|t| (t.title, t.done))
            .collect();
        assert_eq!(
            rows,
            [("b".into(), false), ("c".into(), false), ("a".into(), true)]
        );
        s.clear_done().unwrap();
        assert_eq!(s.list().unwrap().len(), 2);
        let id_b = s.list().unwrap()[0].id;
        s.remove(id_b).unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
    }
}
