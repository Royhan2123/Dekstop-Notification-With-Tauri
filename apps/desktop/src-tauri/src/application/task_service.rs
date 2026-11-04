use std::sync::Arc;

use crate::domain::error::TaskError;
use crate::domain::ports::TaskStore;
use crate::domain::task::{MAX_TASKS, Task, clean_body, clean_title};

/// Daftar tugas dan catatan sederhana: tambah, centang, ubah jadi catatan, hapus.
pub struct TaskService {
    store: Arc<dyn TaskStore>,
}

impl TaskService {
    pub fn new(store: Arc<dyn TaskStore>) -> Arc<Self> {
        Arc::new(Self { store })
    }

    pub fn list(&self) -> Result<Vec<Task>, TaskError> {
        self.store.list()
    }

    pub fn add(&self, title: &str, note: bool) -> Result<(), TaskError> {
        let title = clean_title(title).ok_or(TaskError::EmptyTitle)?;
        if self.store.list()?.len() >= MAX_TASKS {
            return Err(TaskError::TooMany);
        }
        self.store.add(&title, note)
    }

    /// Tugas menjadi catatan (atau kembali); catatan tidak punya status selesai.
    pub fn set_note(&self, id: i32, note: bool) -> Result<(), TaskError> {
        self.store.set_note(id, note)
    }

    pub fn update_note(&self, id: i32, title: &str, body: &str) -> Result<(), TaskError> {
        let title = clean_title(title).ok_or(TaskError::EmptyTitle)?;
        self.store.update_note(id, &title, &clean_body(body))
    }

    pub fn set_done(&self, id: i32, done: bool) -> Result<(), TaskError> {
        self.store.set_done(id, done)
    }

    pub fn remove(&self, id: i32) -> Result<(), TaskError> {
        self.store.remove(id)
    }

    pub fn clear_done(&self) -> Result<(), TaskError> {
        self.store.clear_done()
    }
}
