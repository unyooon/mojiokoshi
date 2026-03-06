use super::sqlite::SqliteStorage;
use super::types::Bookmark;
use crate::error::AppError;

impl SqliteStorage {
    /// Create the `bookmarks` table if it does not already exist.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Storage` when the SQL statement fails.
    pub(crate) fn init_bookmark_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS bookmarks (\
                id TEXT PRIMARY KEY, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                segment_id TEXT REFERENCES segments(id), \
                note TEXT, \
                created_at TEXT NOT NULL DEFAULT (datetime('now'))\
            );\
            CREATE INDEX IF NOT EXISTS idx_bookmarks_session ON bookmarks(session_id);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))
    }

    /// Add a bookmark for a session, optionally linked to a specific segment.
    ///
    /// # Parameters
    ///
    /// * `session_id` - The session to attach the bookmark to.
    /// * `segment_id` - Optional segment ID to link the bookmark to.
    /// * `note` - Optional freeform note.
    ///
    /// # Returns
    ///
    /// The newly created bookmark's UUID.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Storage` when the insert fails.
    pub fn add_bookmark(
        &self,
        session_id: &str,
        segment_id: Option<&str>,
        note: Option<&str>,
    ) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO bookmarks (id, session_id, segment_id, note, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, session_id, segment_id, note, now],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(id)
    }

    /// Remove a bookmark by its ID.
    ///
    /// # Parameters
    ///
    /// * `id` - UUID of the bookmark to remove.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Storage` when the delete fails or the bookmark is not found.
    pub fn remove_bookmark(&self, id: &str) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!("bookmark not found: {id}")));
        }
        Ok(())
    }

    /// Retrieve all bookmarks for a session, ordered by creation time.
    ///
    /// # Parameters
    ///
    /// * `session_id` - The session whose bookmarks are to be fetched.
    ///
    /// # Returns
    ///
    /// A list of `Bookmark` values ordered by `created_at` ascending.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Storage` when the query fails.
    pub fn get_bookmarks(&self, session_id: &str) -> Result<Vec<Bookmark>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, segment_id, note, created_at \
                 FROM bookmarks WHERE session_id = ?1 ORDER BY created_at",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id], |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    segment_id: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Segment, SessionStorage};
    use super::*;

    fn insert_test_segment(storage: &SqliteStorage, session_id: &str) -> i64 {
        storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.to_string(),
                speaker: None,
                text: "Test".to_string(),
                start_time: 0.0,
                end_time: 1.0,
                confidence: None,
                is_partial: false,
            })
            .unwrap()
    }

    #[test]
    fn add_and_get_bookmark() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("BM Test").unwrap();
        let bm_id = storage
            .add_bookmark(&sid, None, Some("Important moment"))
            .unwrap();
        assert!(!bm_id.is_empty());
        let bookmarks = storage.get_bookmarks(&sid).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].note.as_deref(), Some("Important moment"));
        assert_eq!(bookmarks[0].session_id, sid);
        assert!(bookmarks[0].segment_id.is_none());
    }

    #[test]
    fn add_bookmark_with_segment_id() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("BM Seg").unwrap();
        let row_id = insert_test_segment(&storage, &sid);
        let seg_id_str = row_id.to_string();
        let bm_id = storage.add_bookmark(&sid, Some(&seg_id_str), None).unwrap();
        assert!(!bm_id.is_empty());
        let bookmarks = storage.get_bookmarks(&sid).unwrap();
        assert_eq!(
            bookmarks[0].segment_id.as_deref(),
            Some(seg_id_str.as_str())
        );
        assert!(bookmarks[0].note.is_none());
    }

    #[test]
    fn remove_bookmark() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("BM Remove").unwrap();
        let bm_id = storage.add_bookmark(&sid, None, None).unwrap();
        storage.remove_bookmark(&bm_id).unwrap();
        let bookmarks = storage.get_bookmarks(&sid).unwrap();
        assert!(bookmarks.is_empty());
    }

    #[test]
    fn remove_nonexistent_bookmark_errors() {
        let storage = SqliteStorage::in_memory().unwrap();
        let result = storage.remove_bookmark("no-such-id");
        assert!(result.is_err());
    }

    #[test]
    fn get_bookmarks_empty_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("BM Empty").unwrap();
        let bookmarks = storage.get_bookmarks(&sid).unwrap();
        assert!(bookmarks.is_empty());
    }

    #[test]
    fn get_bookmarks_only_for_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid1 = storage.create_session("BM S1").unwrap();
        let sid2 = storage.create_session("BM S2").unwrap();
        storage.add_bookmark(&sid1, None, Some("note1")).unwrap();
        storage.add_bookmark(&sid2, None, Some("note2")).unwrap();
        let bookmarks = storage.get_bookmarks(&sid1).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].note.as_deref(), Some("note1"));
    }
}
