use std::sync::Mutex;

use rusqlite::Connection;

use super::{Segment, SessionStorage};
use crate::error::AppError;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(path: &str) -> Result<Self, AppError> {
        Self::from_conn(Connection::open(path).map_err(|e| AppError::Storage(e.to_string()))?)
    }

    pub fn in_memory() -> Result<Self, AppError> {
        Self::from_conn(Connection::open_in_memory().map_err(|e| AppError::Storage(e.to_string()))?)
    }

    fn from_conn(conn: Connection) -> Result<Self, AppError> {
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    pub(crate) fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|e| AppError::Storage(format!("lock poisoned: {e}")))
    }

    fn init_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (\
                id TEXT PRIMARY KEY, title TEXT, started_at TEXT NOT NULL, \
                ended_at TEXT, target_app TEXT, whisper_model TEXT, \
                status TEXT DEFAULT 'active');\
            CREATE TABLE IF NOT EXISTS segments (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                speaker TEXT, text TEXT NOT NULL, start_time REAL NOT NULL, \
                end_time REAL NOT NULL, confidence REAL, \
                is_partial INTEGER DEFAULT 0, \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP);\
            CREATE INDEX IF NOT EXISTS idx_segments_session ON segments(session_id);\
            CREATE INDEX IF NOT EXISTS idx_segments_time ON segments(session_id, start_time);\
            CREATE TABLE IF NOT EXISTS keywords (\
                id TEXT PRIMARY KEY, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                term TEXT NOT NULL, type TEXT NOT NULL, definition TEXT, \
                web_search_result TEXT, source_url TEXT, \
                first_seen_at REAL NOT NULL, occurrences INTEGER DEFAULT 1);\
            CREATE INDEX IF NOT EXISTS idx_keywords_session ON keywords(session_id);\
            CREATE UNIQUE INDEX IF NOT EXISTS idx_keywords_term_session \
                ON keywords(session_id, term);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        // Release the connection lock before calling sub-init methods
        // which also acquire it.
        drop(conn);
        self.init_speaker_tables()?;
        self.init_search_tables()?;
        self.init_settings_tables()?;
        Ok(())
    }

    pub fn insert_segment(&self, segment: &Segment) -> Result<i64, AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO segments (session_id, speaker, text, \
             start_time, end_time, confidence, is_partial) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                segment.session_id,
                segment.speaker,
                segment.text,
                segment.start_time,
                segment.end_time,
                segment.confidence,
                segment.is_partial,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_segments(&self, session_id: &str) -> Result<Vec<Segment>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, speaker, text, start_time, \
                 end_time, confidence, is_partial \
                 FROM segments WHERE session_id = ?1 ORDER BY start_time",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let segments = stmt
            .query_map(rusqlite::params![session_id], |row| {
                Ok(Segment {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    speaker: row.get(2)?,
                    text: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    confidence: row.get(6)?,
                    is_partial: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(segments)
    }

    /// Retrieve a single session by ID.
    pub fn get_session(&self, session_id: &str) -> Result<super::Session, AppError> {
        let conn = self.lock_conn()?;
        conn.query_row(
            "SELECT id, title, started_at, ended_at, target_app, whisper_model, status \
             FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |row| {
                let status_str: String = row.get(6)?;
                let status = match status_str.as_str() {
                    "completed" => super::SessionStatus::Completed,
                    "archived" => super::SessionStatus::Archived,
                    _ => super::SessionStatus::Active,
                };
                Ok(super::Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    target_app: row.get(4)?,
                    whisper_model: row.get(5)?,
                    status,
                })
            },
        )
        .map_err(|e| AppError::Storage(format!("session not found: {e}")))
    }
}

impl SessionStorage for SqliteStorage {
    fn create_session(&self, title: &str) -> Result<String, AppError> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sessions (id, title, started_at, status) \
             VALUES (?1, ?2, ?3, 'active')",
            rusqlite::params![id, title, now],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(id)
    }

    fn end_session(&self, session_id: &str) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE sessions SET ended_at = ?1, status = 'completed' \
                 WHERE id = ?2",
                rusqlite::params![now, session_id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!(
                "session not found: {session_id}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_end_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let id = storage.create_session("Test Meeting").unwrap();
        assert!(!id.is_empty());
        storage.end_session(&id).unwrap();
    }

    #[test]
    fn insert_and_get_segments() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Test").unwrap();
        let segment = Segment {
            id: 0,
            session_id: session_id.clone(),
            speaker: Some("Alice".to_string()),
            text: "Hello world".to_string(),
            start_time: 0.0,
            end_time: 1.5,
            confidence: Some(0.95),
            is_partial: false,
        };
        let row_id = storage.insert_segment(&segment).unwrap();
        assert!(row_id > 0);
        let segments = storage.get_segments(&session_id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");
    }

    #[test]
    fn get_segments_empty_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Empty").unwrap();
        let segments = storage.get_segments(&session_id).unwrap();
        assert!(segments.is_empty());
    }

    #[test]
    fn segments_ordered_by_start_time() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Ordered").unwrap();
        for (i, start) in [2.0, 0.5, 1.0].iter().enumerate() {
            storage
                .insert_segment(&Segment {
                    id: 0,
                    session_id: session_id.clone(),
                    speaker: None,
                    text: format!("Segment {i}"),
                    start_time: *start,
                    end_time: start + 0.5,
                    confidence: None,
                    is_partial: false,
                })
                .unwrap();
        }
        let segments = storage.get_segments(&session_id).unwrap();
        assert_eq!(segments.len(), 3);
        assert!(segments[0].start_time < segments[1].start_time);
    }

    #[test]
    fn end_nonexistent_session_errors() {
        let storage = SqliteStorage::in_memory().unwrap();
        let result = storage.end_session("nonexistent-id");
        assert!(result.is_err());
    }
}
