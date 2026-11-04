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
            .prepare("SELECT id, title, done, note, body FROM tasks ORDER BY done, id")
            .map_err(err)?;
        q.query_map([], |r| {
            Ok(Task {
                id: r.get(0)?,
                title: r.get(1)?,
                done: r.get(2)?,
                note: r.get(3)?,
                body: r.get(4)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    fn add(&self, title: &str, note: bool) -> Result<(), TaskError> {
        self.0
            .conn()
            .execute(
                "INSERT INTO tasks (title, done, note) VALUES (?1, 0, ?2)",
                rusqlite::params![title, note],
            )
            .map(|_| ())
            .map_err(err)
    }

    /// Catatan selalu belum "selesai" supaya tidak tenggelam di bawah tugas yang dicentang.
    fn set_note(&self, id: i32, note: bool) -> Result<(), TaskError> {
        let n = self
            .0
            .conn()
            .execute(
                "UPDATE tasks SET note = ?2, done = 0 WHERE id = ?1",
                rusqlite::params![id, note],
            )
            .map_err(err)?;
        if n == 0 {
            Err(TaskError::NotFound)
        } else {
            Ok(())
        }
    }

    fn update_note(&self, id: i32, title: &str, body: &str) -> Result<(), TaskError> {
        let n = self
            .0
            .conn()
            .execute(
                "UPDATE tasks SET title = ?2, body = ?3 WHERE id = ?1 AND note = 1",
                rusqlite::params![id, title, body],
            )
            .map_err(err)?;
        if n == 0 {
            Err(TaskError::NotFound)
        } else {
            Ok(())
        }
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
        s.add("a", false).unwrap();
        s.add("b", false).unwrap();
        s.add("c", false).unwrap();
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

    #[test]
    fn tugas_jadi_catatan_isinya_disimpan_dan_bisa_kembali() {
        let s = SqliteTaskStore(Arc::new(Database::open_in_memory().unwrap()));
        s.add("rapat", false).unwrap();
        let id = s.list().unwrap()[0].id;
        s.set_done(id, true).unwrap();
        // Tugas biasa tidak boleh menerima isi catatan.
        assert!(matches!(
            s.update_note(id, "rapat", "x"),
            Err(TaskError::NotFound)
        ));
        s.set_note(id, true).unwrap();
        s.update_note(id, "Rapat Senin", "bawa laptop\nsiapkan slide")
            .unwrap();
        let n = &s.list().unwrap()[0];
        assert!(n.note && !n.done);
        assert_eq!(
            (n.title.as_str(), n.body.as_str()),
            ("Rapat Senin", "bawa laptop\nsiapkan slide")
        );
        s.set_note(id, false).unwrap();
        let t = &s.list().unwrap()[0];
        assert!(!t.note);
        assert_eq!(t.body, "bawa laptop\nsiapkan slide");
        assert!(matches!(s.set_note(999, true), Err(TaskError::NotFound)));
    }
}
