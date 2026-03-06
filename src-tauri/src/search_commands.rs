use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::storage::search_store::SearchResult;
use crate::storage::sqlite::SqliteStorage;

/// Rebuild the FTS5 search index for a specific session.
///
/// # Parameters
///
/// * `storage` - Injected SQLite storage state.
/// * `session_id` - The session whose FTS index should be rebuilt.
///
/// # Errors
///
/// Returns `AppError::Storage` when the index rebuild fails.
#[tauri::command]
#[specta::specta]
pub fn rebuild_search_index(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<(), AppError> {
    storage.rebuild_fts_index(&session_id)
}

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
