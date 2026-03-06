use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::types::Bookmark;

/// Add a bookmark for a session, optionally linked to a specific segment.
///
/// # Parameters
///
/// * `storage` - Injected SQLite storage state.
/// * `session_id` - The session to attach the bookmark to.
/// * `segment_id` - Optional segment ID to link the bookmark to.
/// * `note` - Optional freeform note.
///
/// # Returns
///
/// The UUID of the newly created bookmark.
///
/// # Errors
///
/// Returns `AppError::Storage` when the insert fails.
#[tauri::command]
#[specta::specta]
pub fn add_bookmark(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
    segment_id: Option<String>,
    note: Option<String>,
) -> Result<String, AppError> {
    storage.add_bookmark(&session_id, segment_id.as_deref(), note.as_deref())
}

/// Remove a bookmark by its ID.
///
/// # Parameters
///
/// * `storage` - Injected SQLite storage state.
/// * `id` - UUID of the bookmark to remove.
///
/// # Errors
///
/// Returns `AppError::Storage` when the bookmark is not found or the delete fails.
#[tauri::command]
#[specta::specta]
pub fn remove_bookmark(
    storage: State<'_, Arc<SqliteStorage>>,
    id: String,
) -> Result<(), AppError> {
    storage.remove_bookmark(&id)
}

/// Retrieve all bookmarks for a session.
///
/// # Parameters
///
/// * `storage` - Injected SQLite storage state.
/// * `session_id` - The session whose bookmarks should be retrieved.
///
/// # Returns
///
/// A list of bookmarks ordered by creation time ascending.
///
/// # Errors
///
/// Returns `AppError::Storage` when the query fails.
#[tauri::command]
#[specta::specta]
pub fn get_bookmarks(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<Bookmark>, AppError> {
    storage.get_bookmarks(&session_id)
}
