use std::sync::{Arc, Mutex};

use crate::domain::chat::{ChatTurn, MAX_CHAT_TURNS};
use crate::domain::error::ChatError;
use crate::domain::ports::ChatStore;

use super::sqlite::Database;

pub struct SqliteChatStore(pub Arc<Database>);

fn err(e: rusqlite::Error) -> ChatError {
    ChatError::Storage(e.to_string())
}

impl ChatStore for SqliteChatStore {
    fn list(&self) -> Result<Vec<ChatTurn>, ChatError> {
        let conn = self.0.conn();
        let mut q = conn
            .prepare("SELECT question, answer, model FROM chat_turns ORDER BY id")
            .map_err(err)?;
        q.query_map([], |r| {
            Ok(ChatTurn {
                question: r.get(0)?,
                answer: r.get(1)?,
                by: r.get(2)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    fn add(&self, turn: &ChatTurn) -> Result<(), ChatError> {
        let conn = self.0.conn();
        conn.execute(
            "INSERT INTO chat_turns (question, answer, model) VALUES (?1, ?2, ?3)",
            rusqlite::params![turn.question, turn.answer, turn.by],
        )
        .map_err(err)?;
        conn.execute(
            "DELETE FROM chat_turns WHERE id NOT IN (SELECT id FROM chat_turns ORDER BY id DESC LIMIT ?1)",
            [MAX_CHAT_TURNS as i64],
        )
        .map(|_| ())
        .map_err(err)
    }

    fn clear(&self) -> Result<(), ChatError> {
        self.0
            .conn()
            .execute("DELETE FROM chat_turns", [])
            .map(|_| ())
            .map_err(err)
    }
}

/// Cadangan bila database tidak bisa dibuka: riwayat hidup sampai aplikasi ditutup.
#[derive(Default)]
pub struct MemoryChatStore(Mutex<Vec<ChatTurn>>);

impl ChatStore for MemoryChatStore {
    fn list(&self) -> Result<Vec<ChatTurn>, ChatError> {
        Ok(self.0.lock().unwrap_or_else(|e| e.into_inner()).clone())
    }

    fn add(&self, turn: &ChatTurn) -> Result<(), ChatError> {
        let mut turns = self.0.lock().unwrap_or_else(|e| e.into_inner());
        turns.push(turn.clone());
        let extra = turns.len().saturating_sub(MAX_CHAT_TURNS);
        turns.drain(..extra);
        Ok(())
    }

    fn clear(&self) -> Result<(), ChatError> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(n: usize) -> ChatTurn {
        ChatTurn {
            question: format!("q{n}"),
            answer: format!("a{n}"),
            by: "Claude".into(),
        }
    }

    fn check(store: &dyn ChatStore) {
        assert!(store.list().unwrap().is_empty());
        store.add(&turn(1)).unwrap();
        store.add(&turn(2)).unwrap();
        assert_eq!(store.list().unwrap(), [turn(1), turn(2)], "terlama di atas");
        for n in 3..=MAX_CHAT_TURNS + 5 {
            store.add(&turn(n)).unwrap();
        }
        let all = store.list().unwrap();
        assert_eq!(all.len(), MAX_CHAT_TURNS);
        assert_eq!(all[0], turn(6), "yang terlama dibuang");
        assert_eq!(all.last().unwrap(), &turn(MAX_CHAT_TURNS + 5));
        store.clear().unwrap();
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn sqlite_menyimpan_membatasi_dan_menghapus() {
        check(&SqliteChatStore(Arc::new(
            Database::open_in_memory().unwrap(),
        )));
    }

    #[test]
    fn cadangan_memori_berperilaku_sama() {
        check(&MemoryChatStore::default());
    }

    #[test]
    fn riwayat_bertahan_setelah_database_dibuka_ulang() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("isla.db");
        SqliteChatStore(Arc::new(Database::open(&path).unwrap()))
            .add(&turn(1))
            .unwrap();
        let again = SqliteChatStore(Arc::new(Database::open(&path).unwrap()));
        assert_eq!(again.list().unwrap(), [turn(1)]);
    }
}
