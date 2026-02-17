use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::storage::search_store::SearchResult;
use crate::storage::sqlite::SqliteStorage;

#[tauri::command]
#[specta::specta]
pub fn search_transcripts(
    storage: State<'_, Arc<SqliteStorage>>,
    query: String,
    session_id: Option<String>,
) -> Result<Vec<SearchResult>, AppError> {
    storage.search_segments(&query, session_id.as_deref())
}

#[tauri::command]
#[specta::specta]
pub fn get_setting(
    storage: State<'_, Arc<SqliteStorage>>,
    key: String,
) -> Result<Option<String>, AppError> {
    storage.get_setting(&key)
}

#[tauri::command]
#[specta::specta]
pub fn set_setting(
    storage: State<'_, Arc<SqliteStorage>>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    storage.set_setting(&key, &value)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_settings(
    storage: State<'_, Arc<SqliteStorage>>,
) -> Result<HashMap<String, String>, AppError> {
    storage.get_all_settings()
}
