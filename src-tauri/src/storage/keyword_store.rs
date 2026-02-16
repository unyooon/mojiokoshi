use crate::claude::types::{Keyword, KeywordType};
use crate::error::AppError;

use super::sqlite::SqliteStorage;

impl SqliteStorage {
    pub fn insert_keyword(&self, session_id: &str, kw: &Keyword) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let kw_type = format!("{:?}", kw.keyword_type);
        conn.execute(
            "INSERT OR IGNORE INTO keywords \
             (id, session_id, term, type, definition, web_search_result, \
              source_url, first_seen_at, occurrences) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                kw.id,
                session_id,
                kw.term,
                kw_type,
                kw.definition,
                kw.web_search_result,
                kw.source_url,
                kw.first_seen_at,
                kw.occurrences,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_keywords(&self, session_id: &str) -> Result<Vec<Keyword>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, term, type, definition, web_search_result, \
                 source_url, first_seen_at, occurrences \
                 FROM keywords WHERE session_id = ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id], |row| {
                let kw_type_str: String = row.get(2)?;
                Ok(Keyword {
                    id: row.get(0)?,
                    term: row.get(1)?,
                    keyword_type: KeywordType::from_db_str(&kw_type_str),
                    definition: row.get(3)?,
                    web_search_result: row.get(4)?,
                    source_url: row.get(5)?,
                    first_seen_at: row.get(6)?,
                    occurrences: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }

    pub fn get_keyword_terms(&self, session_id: &str) -> Result<Vec<String>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare("SELECT term FROM keywords WHERE session_id = ?1")
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let terms = stmt
            .query_map(rusqlite::params![session_id], |row| row.get(0))
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(terms)
    }

    pub fn increment_keyword_occurrence(
        &self,
        session_id: &str,
        term: &str,
    ) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE keywords SET occurrences = occurrences + 1 \
             WHERE session_id = ?1 AND term = ?2 COLLATE NOCASE",
            rusqlite::params![session_id, term],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionStorage;
    use super::*;

    #[test]
    fn insert_and_get_keywords() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("KW Test").unwrap();
        let kw = Keyword {
            id: "kw-1".to_string(),
            term: "Rust".to_string(),
            keyword_type: KeywordType::TechTerm,
            definition: Some("A systems language".to_string()),
            web_search_result: None,
            source_url: None,
            first_seen_at: 1000.0,
            occurrences: 1,
        };
        storage.insert_keyword(&sid, &kw).unwrap();
        let terms = storage.get_keyword_terms(&sid).unwrap();
        assert_eq!(terms, vec!["Rust"]);
        let keywords = storage.get_keywords(&sid).unwrap();
        assert_eq!(keywords.len(), 1);
        assert_eq!(keywords[0].term, "Rust");
        assert!(matches!(keywords[0].keyword_type, KeywordType::TechTerm));
    }

    #[test]
    fn increment_keyword_occurrence() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("KW Inc").unwrap();
        let kw = Keyword {
            id: "kw-2".to_string(),
            term: "WebRTC".to_string(),
            keyword_type: KeywordType::Acronym,
            definition: None,
            web_search_result: None,
            source_url: None,
            first_seen_at: 500.0,
            occurrences: 1,
        };
        storage.insert_keyword(&sid, &kw).unwrap();
        storage
            .increment_keyword_occurrence(&sid, "WebRTC")
            .unwrap();
        let conn = storage.lock_conn().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT occurrences FROM keywords WHERE session_id = ?1 AND term = ?2",
                rusqlite::params![sid, "WebRTC"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn duplicate_keyword_insert_ignored() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Dup").unwrap();
        let kw = Keyword {
            id: "kw-3".to_string(),
            term: "Docker".to_string(),
            keyword_type: KeywordType::ProperNoun,
            definition: None,
            web_search_result: None,
            source_url: None,
            first_seen_at: 200.0,
            occurrences: 1,
        };
        storage.insert_keyword(&sid, &kw).unwrap();
        let kw2 = Keyword {
            id: "kw-3b".to_string(),
            ..kw.clone()
        };
        storage.insert_keyword(&sid, &kw2).unwrap();
        let terms = storage.get_keyword_terms(&sid).unwrap();
        assert_eq!(terms.len(), 1);
    }

    #[test]
    fn get_keywords_deserializes_all_types() {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Types").unwrap();
        let types = [
            ("kw-t", "Tauri", KeywordType::TechTerm),
            ("kw-p", "Google", KeywordType::ProperNoun),
            ("kw-a", "API", KeywordType::Acronym),
            ("kw-j", "standup", KeywordType::Jargon),
        ];
        for (id, term, kt) in &types {
            storage
                .insert_keyword(
                    &sid,
                    &Keyword {
                        id: id.to_string(),
                        term: term.to_string(),
                        keyword_type: kt.clone(),
                        definition: None,
                        web_search_result: None,
                        source_url: None,
                        first_seen_at: 0.0,
                        occurrences: 1,
                    },
                )
                .unwrap();
        }
        let keywords = storage.get_keywords(&sid).unwrap();
        assert_eq!(keywords.len(), 4);
    }
}
