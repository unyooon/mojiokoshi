use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;

use super::sqlite::SqliteStorage;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MeetingLink {
    pub id: i64,
    pub session_id: String,
    pub related_session_id: String,
    pub related_title: String,
    pub similarity_score: f64,
    pub shared_keywords: Vec<String>,
    pub created_at: String,
}

impl SqliteStorage {
    pub(crate) fn init_meeting_link_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meeting_links (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                related_session_id TEXT NOT NULL REFERENCES sessions(id), \
                related_title TEXT NOT NULL, \
                similarity_score REAL NOT NULL, \
                shared_keywords TEXT NOT NULL, \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP);\
            CREATE INDEX IF NOT EXISTS idx_meeting_links_session \
                ON meeting_links(session_id);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn insert_meeting_link(&self, link: &MeetingLink) -> Result<i64, AppError> {
        let conn = self.lock_conn()?;
        let shared_json = serde_json::to_string(&link.shared_keywords)
            .map_err(|e| AppError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO meeting_links \
             (session_id, related_session_id, related_title, \
              similarity_score, shared_keywords) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                link.session_id,
                link.related_session_id,
                link.related_title,
                link.similarity_score,
                shared_json,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_meeting_links(&self, session_id: &str) -> Result<Vec<MeetingLink>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, related_session_id, related_title, \
                 similarity_score, shared_keywords, created_at \
                 FROM meeting_links WHERE session_id = ?1 \
                 ORDER BY similarity_score DESC",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id], |row| {
                let keywords_json: String = row.get(5)?;
                let shared_keywords: Vec<String> =
                    serde_json::from_str(&keywords_json).unwrap_or_default();
                Ok(MeetingLink {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    related_session_id: row.get(2)?,
                    related_title: row.get(3)?,
                    similarity_score: row.get(4)?,
                    shared_keywords,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }

    pub fn find_related_meetings(&self, session_id: &str) -> Result<Vec<MeetingLink>, AppError> {
        let my_terms: HashSet<String> = self
            .get_keyword_terms(session_id)?
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect();

        if my_terms.is_empty() {
            return Ok(Vec::new());
        }

        let other_sessions = self.other_sessions_with_keywords(session_id)?;
        let my_count = my_terms.len();

        // Clear old links for this session before recalculating
        {
            let conn = self.lock_conn()?;
            conn.execute(
                "DELETE FROM meeting_links WHERE session_id = ?1",
                rusqlite::params![session_id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        }

        let mut links = Vec::new();
        for (other_id, other_title) in &other_sessions {
            let other_terms: HashSet<String> = self
                .get_keyword_terms(other_id)?
                .into_iter()
                .map(|t| t.to_lowercase())
                .collect();
            let shared: Vec<String> = my_terms.intersection(&other_terms).cloned().collect();
            let max_count = my_count.max(other_terms.len());
            if max_count == 0 {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let score = shared.len() as f64 / max_count as f64;
            if score > 0.1 {
                let link = MeetingLink {
                    id: 0,
                    session_id: session_id.to_string(),
                    related_session_id: other_id.clone(),
                    related_title: other_title.clone(),
                    similarity_score: score,
                    shared_keywords: shared,
                    created_at: String::new(),
                };
                self.insert_meeting_link(&link)?;
                links.push(link);
            }
        }

        links.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(links)
    }

    /// Return (session_id, title) pairs for all sessions other than the
    /// given one that have at least one keyword.
    fn other_sessions_with_keywords(
        &self,
        exclude_id: &str,
    ) -> Result<Vec<(String, String)>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT s.id, s.title FROM sessions s \
                 INNER JOIN keywords k ON k.session_id = s.id \
                 WHERE s.id != ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![exclude_id], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                Ok((id, title))
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionStorage;
    use super::*;
    use crate::claude::types::{Keyword, KeywordType};

    fn make_keyword(id: &str, term: &str) -> Keyword {
        Keyword {
            id: id.to_string(),
            term: term.to_string(),
            keyword_type: KeywordType::TechTerm,
            definition: None,
            web_search_result: None,
            source_url: None,
            first_seen_at: 0.0,
            occurrences: 1,
        }
    }

    #[test]
    fn insert_and_get_meeting_link() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("Meeting A").unwrap();
        let s2 = storage.create_session("Meeting B").unwrap();

        let link = MeetingLink {
            id: 0,
            session_id: s1.clone(),
            related_session_id: s2.clone(),
            related_title: "Meeting B".to_string(),
            similarity_score: 0.75,
            shared_keywords: vec!["Rust".to_string(), "Tauri".to_string()],
            created_at: String::new(),
        };
        let row_id = storage.insert_meeting_link(&link).unwrap();
        assert!(row_id > 0);

        let links = storage.get_meeting_links(&s1).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].related_session_id, s2);
        assert_eq!(links[0].shared_keywords.len(), 2);
    }

    #[test]
    fn find_related_meetings_with_overlap() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("Session 1").unwrap();
        let s2 = storage.create_session("Session 2").unwrap();

        // Both sessions share "Rust" and "Tauri"
        for kw in &[
            make_keyword("k1", "Rust"),
            make_keyword("k2", "Tauri"),
            make_keyword("k3", "WebRTC"),
        ] {
            storage.insert_keyword(&s1, kw).unwrap();
        }
        for kw in &[
            make_keyword("k4", "Rust"),
            make_keyword("k5", "Tauri"),
            make_keyword("k6", "Docker"),
        ] {
            storage.insert_keyword(&s2, kw).unwrap();
        }

        let links = storage.find_related_meetings(&s1).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].related_session_id, s2);
        // 2 shared / max(3, 3) = 0.666…
        assert!(links[0].similarity_score > 0.6);
        assert!(links[0].similarity_score < 0.7);
        assert_eq!(links[0].shared_keywords.len(), 2);
    }

    #[test]
    fn find_related_meetings_no_overlap() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("Session A").unwrap();
        let s2 = storage.create_session("Session B").unwrap();

        storage
            .insert_keyword(&s1, &make_keyword("k1", "Rust"))
            .unwrap();
        storage
            .insert_keyword(&s2, &make_keyword("k2", "Python"))
            .unwrap();

        let links = storage.find_related_meetings(&s1).unwrap();
        assert!(links.is_empty());
    }

    #[test]
    fn find_related_meetings_no_keywords() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("Empty").unwrap();
        let links = storage.find_related_meetings(&s1).unwrap();
        assert!(links.is_empty());
    }

    #[test]
    fn find_related_meetings_case_insensitive() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("S1").unwrap();
        let s2 = storage.create_session("S2").unwrap();

        storage
            .insert_keyword(&s1, &make_keyword("k1", "Rust"))
            .unwrap();
        storage
            .insert_keyword(&s2, &make_keyword("k2", "rust"))
            .unwrap();

        let links = storage.find_related_meetings(&s1).unwrap();
        assert_eq!(links.len(), 1);
        assert!((links[0].similarity_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn find_related_recalculates_links() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("S1").unwrap();
        let s2 = storage.create_session("S2").unwrap();

        storage
            .insert_keyword(&s1, &make_keyword("k1", "Rust"))
            .unwrap();
        storage
            .insert_keyword(&s2, &make_keyword("k2", "Rust"))
            .unwrap();

        // Run twice; should not duplicate links
        storage.find_related_meetings(&s1).unwrap();
        storage.find_related_meetings(&s1).unwrap();

        let links = storage.get_meeting_links(&s1).unwrap();
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn get_meeting_links_empty() {
        let storage = SqliteStorage::in_memory().unwrap();
        let s1 = storage.create_session("No links").unwrap();
        let links = storage.get_meeting_links(&s1).unwrap();
        assert!(links.is_empty());
    }
}
