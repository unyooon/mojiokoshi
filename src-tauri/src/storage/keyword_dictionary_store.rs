use serde::{Deserialize, Serialize};
use specta::Type;

use super::sqlite::SqliteStorage;
use crate::error::AppError;

/// A user-defined keyword with optional reading and definition.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DictionaryKeyword {
    pub id: i64,
    pub term: String,
    /// Japanese reading (furigana).
    pub reading: Option<String>,
    pub definition: Option<String>,
    /// One of "tech_term", "proper_noun", "acronym", or "custom".
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

impl SqliteStorage {
    /// Create the keyword_dictionary table if it does not already exist.
    pub(crate) fn init_keyword_dictionary_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS keyword_dictionary (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                term TEXT NOT NULL UNIQUE, \
                reading TEXT, \
                definition TEXT, \
                category TEXT NOT NULL DEFAULT 'custom', \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP, \
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Add a new keyword to the dictionary and return its id.
    pub fn add_dictionary_keyword(
        &self,
        term: &str,
        reading: Option<&str>,
        definition: Option<&str>,
        category: &str,
    ) -> Result<i64, AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO keyword_dictionary (term, reading, definition, category) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![term, reading, definition, category],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    /// Update an existing dictionary keyword by id.
    pub fn update_dictionary_keyword(
        &self,
        id: i64,
        term: &str,
        reading: Option<&str>,
        definition: Option<&str>,
        category: &str,
    ) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute(
                "UPDATE keyword_dictionary \
                 SET term = ?1, reading = ?2, definition = ?3, \
                     category = ?4, updated_at = CURRENT_TIMESTAMP \
                 WHERE id = ?5",
                rusqlite::params![term, reading, definition, category, id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!(
                "dictionary keyword not found: {id}"
            )));
        }
        Ok(())
    }

    /// Delete a dictionary keyword by id.
    pub fn delete_dictionary_keyword(&self, id: i64) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        let rows = conn
            .execute(
                "DELETE FROM keyword_dictionary WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if rows == 0 {
            return Err(AppError::Storage(format!(
                "dictionary keyword not found: {id}"
            )));
        }
        Ok(())
    }

    /// Retrieve all dictionary keywords ordered by term.
    pub fn get_all_dictionary_keywords(&self) -> Result<Vec<DictionaryKeyword>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, term, reading, definition, category, \
                 created_at, updated_at \
                 FROM keyword_dictionary ORDER BY term",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DictionaryKeyword {
                    id: row.get(0)?,
                    term: row.get(1)?,
                    reading: row.get(2)?,
                    definition: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows)
    }

    /// Search dictionary keywords by term using LIKE matching.
    pub fn search_dictionary(&self, query: &str) -> Result<Vec<DictionaryKeyword>, AppError> {
        let conn = self.lock_conn()?;
        let pattern = format!("%{query}%");
        let mut stmt = conn
            .prepare(
                "SELECT id, term, reading, definition, category, \
                 created_at, updated_at \
                 FROM keyword_dictionary WHERE term LIKE ?1 ORDER BY term",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok(DictionaryKeyword {
                    id: row.get(0)?,
                    term: row.get(1)?,
                    reading: row.get(2)?,
                    definition: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
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
    use super::*;

    #[test]
    fn add_and_get_keyword() {
        let s = SqliteStorage::in_memory().unwrap();
        let id = s
            .add_dictionary_keyword(
                "Rust",
                Some("ラスト"),
                Some("A systems language"),
                "tech_term",
            )
            .unwrap();
        assert!(id > 0);
        let all = s.get_all_dictionary_keywords().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].term, "Rust");
        assert_eq!(all[0].reading.as_deref(), Some("ラスト"));
        assert_eq!(all[0].definition.as_deref(), Some("A systems language"));
        assert_eq!(all[0].category, "tech_term");
    }

    #[test]
    fn update_keyword() {
        let s = SqliteStorage::in_memory().unwrap();
        let id = s
            .add_dictionary_keyword("Tauri", None, None, "custom")
            .unwrap();
        s.update_dictionary_keyword(
            id,
            "Tauri",
            Some("タウリ"),
            Some("Desktop framework"),
            "tech_term",
        )
        .unwrap();
        let all = s.get_all_dictionary_keywords().unwrap();
        assert_eq!(all[0].reading.as_deref(), Some("タウリ"));
        assert_eq!(all[0].definition.as_deref(), Some("Desktop framework"));
        assert_eq!(all[0].category, "tech_term");
    }

    #[test]
    fn update_nonexistent_keyword_errors() {
        let s = SqliteStorage::in_memory().unwrap();
        let result = s.update_dictionary_keyword(999, "Ghost", None, None, "custom");
        assert!(result.is_err());
    }

    #[test]
    fn delete_keyword() {
        let s = SqliteStorage::in_memory().unwrap();
        let id = s
            .add_dictionary_keyword("Temp", None, None, "custom")
            .unwrap();
        assert_eq!(s.get_all_dictionary_keywords().unwrap().len(), 1);
        s.delete_dictionary_keyword(id).unwrap();
        assert!(s.get_all_dictionary_keywords().unwrap().is_empty());
    }

    #[test]
    fn delete_nonexistent_keyword_errors() {
        let s = SqliteStorage::in_memory().unwrap();
        let result = s.delete_dictionary_keyword(999);
        assert!(result.is_err());
    }

    #[test]
    fn search_dictionary() {
        let s = SqliteStorage::in_memory().unwrap();
        s.add_dictionary_keyword("Rust", None, None, "tech_term")
            .unwrap();
        s.add_dictionary_keyword("RustUp", None, None, "tech_term")
            .unwrap();
        s.add_dictionary_keyword("Python", None, None, "tech_term")
            .unwrap();
        let results = s.search_dictionary("Rust").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn unique_constraint() {
        let s = SqliteStorage::in_memory().unwrap();
        s.add_dictionary_keyword("Unique", None, None, "custom")
            .unwrap();
        let result = s.add_dictionary_keyword("Unique", None, None, "custom");
        assert!(result.is_err());
    }

    #[test]
    fn get_all_empty() {
        let s = SqliteStorage::in_memory().unwrap();
        let all = s.get_all_dictionary_keywords().unwrap();
        assert!(all.is_empty());
    }
}
