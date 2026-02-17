use serde::{Deserialize, Serialize};
use specta::Type;

use super::sqlite::SqliteStorage;
use crate::error::AppError;

/// A single sentiment/tone analysis result for a transcript segment.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SentimentEntry {
    pub id: i64,
    pub session_id: String,
    pub segment_id: i64,
    /// Sentiment score from -1.0 (very negative) to 1.0 (very positive).
    pub score: f64,
    /// Detected emotion label: "neutral", "positive", "negative",
    /// "excited", "concerned", or "confused".
    pub emotion: String,
    /// Confidence of the analysis, 0.0 to 1.0.
    pub confidence: f64,
    /// Segment start_time, used for timeline plotting.
    pub timestamp: f64,
    pub created_at: String,
}

impl SqliteStorage {
    /// Create the sentiments table if it does not already exist.
    pub(crate) fn init_sentiment_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sentiments (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                segment_id INTEGER NOT NULL, \
                score REAL NOT NULL, \
                emotion TEXT NOT NULL, \
                confidence REAL NOT NULL DEFAULT 0.8, \
                timestamp REAL NOT NULL, \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP);\
            CREATE INDEX IF NOT EXISTS idx_sentiments_session \
                ON sentiments(session_id);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Insert a sentiment entry and return its auto-generated row id.
    pub fn insert_sentiment(&self, entry: &SentimentEntry) -> Result<i64, AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO sentiments (session_id, segment_id, score, \
             emotion, confidence, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                entry.session_id,
                entry.segment_id,
                entry.score,
                entry.emotion,
                entry.confidence,
                entry.timestamp,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    /// Retrieve all sentiment entries for a session, ordered by timestamp.
    pub fn get_sentiments(&self, session_id: &str) -> Result<Vec<SentimentEntry>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, segment_id, score, emotion, \
                 confidence, timestamp, created_at \
                 FROM sentiments WHERE session_id = ?1 ORDER BY timestamp",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let entries = stmt
            .query_map(rusqlite::params![session_id], |row| {
                Ok(SentimentEntry {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    segment_id: row.get(2)?,
                    score: row.get(3)?,
                    emotion: row.get(4)?,
                    confidence: row.get(5)?,
                    timestamp: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(entries)
    }

    /// Delete all sentiments for a session. Useful before re-analysis.
    pub fn delete_sentiments(&self, session_id: &str) -> Result<usize, AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute(
                "DELETE FROM sentiments WHERE session_id = ?1",
                rusqlite::params![session_id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionStorage;
    use super::*;
    use crate::storage::Segment;

    fn make_entry(session_id: &str, segment_id: i64, score: f64, emotion: &str) -> SentimentEntry {
        SentimentEntry {
            id: 0,
            session_id: session_id.to_string(),
            segment_id,
            score,
            emotion: emotion.to_string(),
            confidence: 0.85,
            timestamp: segment_id as f64 * 1.0,
            created_at: String::new(),
        }
    }

    fn make_segment(session_id: &str, text: &str, start: f64) -> Segment {
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

    #[test]
    fn insert_and_get_sentiment() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Sentiment Test").unwrap();
        let seg_id = s.insert_segment(&make_segment(&sid, "Hello", 0.0)).unwrap();
        let entry = make_entry(&sid, seg_id, 0.5, "positive");
        let row_id = s.insert_sentiment(&entry).unwrap();
        assert!(row_id > 0);
        let results = s.get_sentiments(&sid).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].emotion, "positive");
        assert!((results[0].score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn sentiments_empty_session() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Empty").unwrap();
        let results = s.get_sentiments(&sid).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn sentiments_ordered_by_timestamp() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Ordered").unwrap();
        // Insert in reverse order to verify ordering
        for (i, ts) in [3.0_f64, 1.0, 2.0].iter().enumerate() {
            let mut entry = make_entry(&sid, (i + 1) as i64, 0.0, "neutral");
            entry.timestamp = *ts;
            s.insert_sentiment(&entry).unwrap();
        }
        let results = s.get_sentiments(&sid).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].timestamp < results[1].timestamp);
        assert!(results[1].timestamp < results[2].timestamp);
    }

    #[test]
    fn delete_sentiments() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Delete").unwrap();
        s.insert_sentiment(&make_entry(&sid, 1, 0.3, "positive"))
            .unwrap();
        s.insert_sentiment(&make_entry(&sid, 2, -0.5, "negative"))
            .unwrap();
        assert_eq!(s.get_sentiments(&sid).unwrap().len(), 2);
        let deleted = s.delete_sentiments(&sid).unwrap();
        assert_eq!(deleted, 2);
        assert!(s.get_sentiments(&sid).unwrap().is_empty());
    }

    #[test]
    fn delete_sentiments_empty_session() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("NoSentiments").unwrap();
        let deleted = s.delete_sentiments(&sid).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn multiple_sentiments_per_session() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Multi").unwrap();
        let emotions = ["positive", "neutral", "negative", "excited", "concerned"];
        for (i, emotion) in emotions.iter().enumerate() {
            let entry = make_entry(&sid, (i + 1) as i64, 0.1 * i as f64, emotion);
            s.insert_sentiment(&entry).unwrap();
        }
        let results = s.get_sentiments(&sid).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn sentiment_score_range() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid = s.create_session("Range").unwrap();
        s.insert_sentiment(&make_entry(&sid, 1, -1.0, "negative"))
            .unwrap();
        s.insert_sentiment(&make_entry(&sid, 2, 1.0, "positive"))
            .unwrap();
        let results = s.get_sentiments(&sid).unwrap();
        assert!((results[0].score - (-1.0)).abs() < f64::EPSILON);
        assert!((results[1].score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sentiments_scoped_to_session() {
        let s = SqliteStorage::in_memory().unwrap();
        let sid1 = s.create_session("Session A").unwrap();
        let sid2 = s.create_session("Session B").unwrap();
        s.insert_sentiment(&make_entry(&sid1, 1, 0.5, "positive"))
            .unwrap();
        s.insert_sentiment(&make_entry(&sid2, 1, -0.5, "negative"))
            .unwrap();
        assert_eq!(s.get_sentiments(&sid1).unwrap().len(), 1);
        assert_eq!(s.get_sentiments(&sid2).unwrap().len(), 1);
        assert_eq!(s.get_sentiments(&sid1).unwrap()[0].emotion, "positive");
        assert_eq!(s.get_sentiments(&sid2).unwrap()[0].emotion, "negative");
    }
}
