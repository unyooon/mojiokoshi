use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;

use super::sqlite::SqliteStorage;

/// A persisted translation of a transcript segment.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TranslationEntry {
    pub id: i64,
    pub session_id: String,
    pub segment_id: i64,
    pub source_lang: String,
    pub target_lang: String,
    pub source_text: String,
    pub translated_text: String,
    pub created_at: String,
}

impl SqliteStorage {
    /// Create the translations table and its index.
    pub(crate) fn init_translation_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS translations (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                session_id TEXT NOT NULL REFERENCES sessions(id), \
                segment_id INTEGER NOT NULL, \
                source_lang TEXT NOT NULL, \
                target_lang TEXT NOT NULL, \
                source_text TEXT NOT NULL, \
                translated_text TEXT NOT NULL, \
                created_at TEXT DEFAULT CURRENT_TIMESTAMP);\
            CREATE INDEX IF NOT EXISTS idx_translations_session \
                ON translations(session_id, target_lang);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Insert a single translation row and return its row id.
    pub fn insert_translation(
        &self,
        session_id: &str,
        segment_id: i64,
        source_lang: &str,
        target_lang: &str,
        source_text: &str,
        translated_text: &str,
    ) -> Result<i64, AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO translations \
             (session_id, segment_id, source_lang, target_lang, \
              source_text, translated_text) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                session_id,
                segment_id,
                source_lang,
                target_lang,
                source_text,
                translated_text,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    /// Return all translations for a session filtered by target language.
    pub fn get_translations(
        &self,
        session_id: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslationEntry>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, segment_id, source_lang, target_lang, \
                 source_text, translated_text, created_at \
                 FROM translations \
                 WHERE session_id = ?1 AND target_lang = ?2 \
                 ORDER BY segment_id",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id, target_lang], |row| {
                Ok(TranslationEntry {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    segment_id: row.get(2)?,
                    source_lang: row.get(3)?,
                    target_lang: row.get(4)?,
                    source_text: row.get(5)?,
                    translated_text: row.get(6)?,
                    created_at: row.get(7)?,
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
    use super::super::SessionStorage;
    use super::*;

    fn setup() -> (SqliteStorage, String) {
        let storage = SqliteStorage::in_memory().unwrap();
        let sid = storage.create_session("Translation Test").unwrap();
        // Insert a segment so that segment_id references are realistic.
        let seg = crate::storage::Segment {
            id: 0,
            session_id: sid.clone(),
            speaker: Some("Alice".into()),
            text: "Hello world".into(),
            start_time: 0.0,
            end_time: 1.0,
            confidence: None,
            is_partial: false,
        };
        storage.insert_segment(&seg).unwrap();
        (storage, sid)
    }

    #[test]
    fn insert_and_get_translation() {
        let (storage, sid) = setup();
        let row_id = storage
            .insert_translation(&sid, 1, "en", "ja", "Hello world", "こんにちは世界")
            .unwrap();
        assert!(row_id > 0);

        let translations = storage.get_translations(&sid, "ja").unwrap();
        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0].source_text, "Hello world");
        assert_eq!(translations[0].translated_text, "こんにちは世界");
        assert_eq!(translations[0].source_lang, "en");
        assert_eq!(translations[0].target_lang, "ja");
        assert_eq!(translations[0].segment_id, 1);
    }

    #[test]
    fn get_translations_filters_by_target_lang() {
        let (storage, sid) = setup();
        storage
            .insert_translation(&sid, 1, "en", "ja", "Hello", "こんにちは")
            .unwrap();
        storage
            .insert_translation(&sid, 1, "en", "fr", "Hello", "Bonjour")
            .unwrap();

        let ja = storage.get_translations(&sid, "ja").unwrap();
        assert_eq!(ja.len(), 1);
        assert_eq!(ja[0].translated_text, "こんにちは");

        let fr = storage.get_translations(&sid, "fr").unwrap();
        assert_eq!(fr.len(), 1);
        assert_eq!(fr[0].translated_text, "Bonjour");
    }

    #[test]
    fn get_translations_empty_session() {
        let (storage, sid) = setup();
        let translations = storage.get_translations(&sid, "ja").unwrap();
        assert!(translations.is_empty());
    }

    #[test]
    fn multiple_translations_ordered_by_segment_id() {
        let (storage, sid) = setup();
        // Add a second segment.
        let seg2 = crate::storage::Segment {
            id: 0,
            session_id: sid.clone(),
            speaker: None,
            text: "Goodbye".into(),
            start_time: 1.0,
            end_time: 2.0,
            confidence: None,
            is_partial: false,
        };
        let seg2_id = storage.insert_segment(&seg2).unwrap();

        // Insert in reverse order to verify ordering.
        storage
            .insert_translation(&sid, seg2_id, "en", "ja", "Goodbye", "さようなら")
            .unwrap();
        storage
            .insert_translation(&sid, 1, "en", "ja", "Hello", "こんにちは")
            .unwrap();

        let translations = storage.get_translations(&sid, "ja").unwrap();
        assert_eq!(translations.len(), 2);
        assert!(translations[0].segment_id < translations[1].segment_id);
    }

    #[test]
    fn translation_entry_serialization_roundtrip() {
        let entry = TranslationEntry {
            id: 1,
            session_id: "s1".to_string(),
            segment_id: 10,
            source_lang: "ja".to_string(),
            target_lang: "en".to_string(),
            source_text: "テスト".to_string(),
            translated_text: "Test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TranslationEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.translated_text, "Test");
    }
}
