use std::collections::HashMap;

use super::sqlite::SqliteStorage;
use crate::error::AppError;

impl SqliteStorage {
    /// Create the `app_settings` table for key-value persistence.
    pub(crate) fn init_settings_tables(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_settings (\
                key TEXT PRIMARY KEY, \
                value TEXT NOT NULL, \
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP\
            );",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Retrieve a setting by key, returning `None` when it does not exist.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare("SELECT value FROM app_settings WHERE key = ?1")
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![key])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        match rows.next().map_err(|e| AppError::Storage(e.to_string()))? {
            Some(row) => {
                let value: String = row.get(0).map_err(|e| AppError::Storage(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Insert or update a setting.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) \
             VALUES (?1, ?2, CURRENT_TIMESTAMP) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
             updated_at = excluded.updated_at",
            rusqlite::params![key, value],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Return all settings as a key-value map.
    pub fn get_all_settings(&self) -> Result<HashMap<String, String>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM app_settings")
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| AppError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(rows.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_setting_returns_none_when_missing() {
        let storage = SqliteStorage::in_memory().unwrap();
        let result = storage.get_setting("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn set_and_get_setting() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.set_setting("theme", "dark").unwrap();
        let value = storage.get_setting("theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn set_setting_upserts() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.set_setting("lang", "en").unwrap();
        storage.set_setting("lang", "ja").unwrap();
        let value = storage.get_setting("lang").unwrap();
        assert_eq!(value, Some("ja".to_string()));
    }

    #[test]
    fn get_all_settings() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.set_setting("a", "1").unwrap();
        storage.set_setting("b", "2").unwrap();
        storage.set_setting("c", "3").unwrap();

        let all = storage.get_all_settings().unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all.get("a").map(String::as_str), Some("1"));
        assert_eq!(all.get("b").map(String::as_str), Some("2"));
        assert_eq!(all.get("c").map(String::as_str), Some("3"));
    }

    #[test]
    fn get_all_settings_empty() {
        let storage = SqliteStorage::in_memory().unwrap();
        let all = storage.get_all_settings().unwrap();
        assert!(all.is_empty());
    }
}
