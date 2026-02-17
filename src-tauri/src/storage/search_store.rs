use serde::{Deserialize, Serialize};
use specta::Type;

use super::sqlite::SqliteStorage;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchResult {
    pub segment_id: i64,
    pub session_id: String,
    pub text: String,
    pub highlighted: String,
    pub start_time: f64,
    pub end_time: f64,
    pub speaker: Option<String>,
}

impl SqliteStorage {
    /// Create the FTS5 virtual table and triggers that keep it in sync with
    /// the `segments` table.
    pub(crate) fn init_search_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS segments_fts \
             USING fts5(text, session_id UNINDEXED, content=segments, content_rowid=id);\
             CREATE TRIGGER IF NOT EXISTS segments_ai AFTER INSERT ON segments BEGIN \
               INSERT INTO segments_fts(rowid, text, session_id) \
               VALUES (new.id, new.text, new.session_id); \
             END;\
             CREATE TRIGGER IF NOT EXISTS segments_ad AFTER DELETE ON segments BEGIN \
               INSERT INTO segments_fts(segments_fts, rowid, text, session_id) \
               VALUES ('delete', old.id, old.text, old.session_id); \
             END;\
             CREATE TRIGGER IF NOT EXISTS segments_au AFTER UPDATE ON segments BEGIN \
               INSERT INTO segments_fts(segments_fts, rowid, text, session_id) \
               VALUES ('delete', old.id, old.text, old.session_id); \
               INSERT INTO segments_fts(rowid, text, session_id) \
               VALUES (new.id, new.text, new.session_id); \
             END;",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Full-text search across transcript segments.
    ///
    /// When `session_id` is `Some`, results are scoped to that session.
    /// The `highlighted` field wraps matching terms in `<mark>` tags.
    pub fn search_segments(
        &self,
        query: &str,
        session_id: Option<&str>,
    ) -> Result<Vec<SearchResult>, AppError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.lock_conn()?;
        let base = "SELECT s.id, s.session_id, s.text, s.start_time, s.end_time, s.speaker, \
                    highlight(segments_fts, 0, '<mark>', '</mark>') AS highlighted \
                    FROM segments_fts fts JOIN segments s ON fts.rowid = s.id \
                    WHERE segments_fts MATCH ?1";
        let sql = if session_id.is_some() {
            format!("{base} AND fts.session_id = ?2 ORDER BY s.start_time")
        } else {
            format!("{base} ORDER BY s.start_time")
        };
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(query.to_string())];
        if let Some(sid) = session_id {
            params.push(Box::new(sid.to_string()));
        }
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| &**p).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| AppError::Storage(e.to_string()))?;
        let results = stmt
            .query_map(refs.as_slice(), |row| {
                Ok(SearchResult {
                    segment_id: row.get(0)?,
                    session_id: row.get(1)?,
                    text: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    speaker: row.get(5)?,
                    highlighted: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionStorage;
    use super::*;
    use crate::storage::Segment;

    fn seg(session_id: &str, text: &str, start: f64) -> Segment {
        Segment {
            id: 0,
            session_id: session_id.to_string(),
            speaker: Some("Alice".to_string()),
            text: text.to_string(),
            start_time: start,
            end_time: start + 1.0,
            confidence: None,
            is_partial: false,
        }
    }

    fn insert(storage: &SqliteStorage, session_id: &str, text: &str, start: f64) {
        storage.insert_segment(&seg(session_id, text, start)).unwrap();
    }

    #[test]
    fn fts5_search_finds_matching_segments() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("FTS").unwrap();
        insert(&s, &sid, "Hello world", 0.0);
        insert(&s, &sid, "Goodbye world", 1.0);
        insert(&s, &sid, "Something else", 2.0);
        let r = s.search_segments("world", None).unwrap();
        assert_eq!(r.len(), 2);
        assert!(r[0].highlighted.contains("<mark>"));
    }

    #[test]
    fn fts5_search_highlights_contain_mark_tags() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("HL").unwrap();
        insert(&s, &sid, "Rust is great", 0.0);
        let r = s.search_segments("Rust", None).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].highlighted.contains("<mark>Rust</mark>"));
    }

    #[test]
    fn fts5_search_with_session_filter() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid1 = s.create_session("A").unwrap();
        let sid2 = s.create_session("B").unwrap();
        insert(&s, &sid1, "Hello from A", 0.0);
        insert(&s, &sid2, "Hello from B", 0.0);
        assert_eq!(s.search_segments("Hello", None).unwrap().len(), 2);
        let filtered = s.search_segments("Hello", Some(&sid1)).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].session_id, sid1);
    }

    #[test]
    fn fts5_search_empty_query_returns_empty() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("EQ").unwrap();
        insert(&s, &sid, "Some text here", 0.0);
        assert!(s.search_segments("", None).unwrap().is_empty());
        assert!(s.search_segments("   ", None).unwrap().is_empty());
    }

    #[test]
    fn fts5_search_no_match_returns_empty() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("NM").unwrap();
        insert(&s, &sid, "Hello world", 0.0);
        assert!(s.search_segments("zzzznotfound", None).unwrap().is_empty());
    }

    #[test]
    fn fts5_results_ordered_by_start_time() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Ord").unwrap();
        insert(&s, &sid, "meeting notes", 3.0);
        insert(&s, &sid, "meeting agenda", 1.0);
        insert(&s, &sid, "meeting summary", 5.0);
        let r = s.search_segments("meeting", None).unwrap();
        assert_eq!(r.len(), 3);
        assert!(r[0].start_time < r[1].start_time);
        assert!(r[1].start_time < r[2].start_time);
    }
}
