use crate::diarization::SpeakerInfo;
use crate::error::AppError;

use super::sqlite::SqliteStorage;

impl SqliteStorage {
    /// Initialize the speakers table and add `speaker_id` column to segments.
    pub(crate) fn init_speaker_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS speakers (\
                id TEXT PRIMARY KEY, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                label TEXT NOT NULL, color TEXT NOT NULL, \
                is_self INTEGER DEFAULT 0, \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP);\
            CREATE INDEX IF NOT EXISTS idx_speakers_session \
                ON speakers(session_id);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        // ALTER TABLE fails if the column already exists; ignore that error.
        if let Err(e) = conn.execute_batch("ALTER TABLE segments ADD COLUMN speaker_id TEXT;") {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") {
                return Err(AppError::Storage(msg));
            }
        }
        Ok(())
    }

    /// Insert a new speaker record and return its generated ID.
    pub fn insert_speaker(
        &self,
        session_id: &str,
        label: &str,
        color: &str,
    ) -> Result<String, AppError> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO speakers (id, session_id, label, color) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, session_id, label, color],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(id)
    }

    /// Return all speakers belonging to a session.
    pub fn get_speakers(&self, session_id: &str) -> Result<Vec<SpeakerInfo>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label, color, is_self \
                 FROM speakers WHERE session_id = ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id], |row| {
                let is_self_int: i32 = row.get(3)?;
                Ok(SpeakerInfo {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    color: row.get(2)?,
                    is_self: is_self_int != 0,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }

    /// Rename a speaker.
    pub fn update_speaker_label(&self, id: &str, label: &str) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute(
                "UPDATE speakers SET label = ?1 WHERE id = ?2",
                rusqlite::params![label, id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!("speaker not found: {id}")));
        }
        Ok(())
    }

    /// Assign a speaker to a transcript segment.
    pub fn update_segment_speaker(
        &self,
        segment_id: i64,
        speaker_id: &str,
    ) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute(
                "UPDATE segments SET speaker_id = ?1 WHERE id = ?2",
                rusqlite::params![speaker_id, segment_id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!(
                "segment not found: {segment_id}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionStorage;
    use super::*;

    #[test]
    fn insert_and_get_speakers() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Speaker Test").unwrap();
        let id = storage.insert_speaker(&sid, "Alice", "#ff0000").unwrap();
        assert!(!id.is_empty());
        let speakers = storage.get_speakers(&sid).unwrap();
        assert_eq!(speakers.len(), 1);
        assert_eq!(speakers[0].label, "Alice");
        assert_eq!(speakers[0].color, "#ff0000");
        assert!(!speakers[0].is_self);
    }

    #[test]
    fn update_speaker_label() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Rename").unwrap();
        let id = storage
            .insert_speaker(&sid, "Speaker_0", "#00ff00")
            .unwrap();
        storage.update_speaker_label(&id, "Bob").unwrap();
        let speakers = storage.get_speakers(&sid).unwrap();
        assert_eq!(speakers[0].label, "Bob");
    }

    #[test]
    fn update_speaker_label_not_found() {
        let storage = SqliteStorage::in_memory().unwrap();
        let result = storage.update_speaker_label("nonexistent", "X");
        assert!(result.is_err());
    }

    #[test]
    fn update_segment_speaker() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Assign").unwrap();
        let spk_id = storage.insert_speaker(&sid, "Alice", "#ff0000").unwrap();
        let seg = crate::storage::Segment {
            id: 0,
            session_id: sid,
            speaker: Some("Unknown".into()),
            text: "Hello".into(),
            start_time: 0.0,
            end_time: 1.0,
            confidence: None,
            is_partial: false,
        };
        let seg_id = storage.insert_segment(&seg).unwrap();
        storage.update_segment_speaker(seg_id, &spk_id).unwrap();
        let conn = storage.lock_conn().unwrap();
        let stored: String = conn
            .query_row(
                "SELECT speaker_id FROM segments WHERE id = ?1",
                rusqlite::params![seg_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, spk_id);
    }

    #[test]
    fn update_segment_speaker_not_found() {
        let storage = SqliteStorage::in_memory().unwrap();
        let result = storage.update_segment_speaker(999, "sp-1");
        assert!(result.is_err());
    }

    #[test]
    fn speakers_empty_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Empty").unwrap();
        let speakers = storage.get_speakers(&sid).unwrap();
        assert!(speakers.is_empty());
    }

    #[test]
    fn multiple_speakers_per_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Multi").unwrap();
        storage.insert_speaker(&sid, "Alice", "#ff0000").unwrap();
        storage.insert_speaker(&sid, "Bob", "#00ff00").unwrap();
        storage.insert_speaker(&sid, "Charlie", "#0000ff").unwrap();
        let speakers = storage.get_speakers(&sid).unwrap();
        assert_eq!(speakers.len(), 3);
    }
}
