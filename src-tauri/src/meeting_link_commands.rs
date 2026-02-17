use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::storage::meeting_link_store::MeetingLink;
use crate::storage::sqlite::SqliteStorage;

#[tauri::command]
#[specta::specta]
pub fn find_related_meetings(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<MeetingLink>, AppError> {
    storage.find_related_meetings(&session_id)
}

#[tauri::command]
#[specta::specta]
pub fn get_meeting_links(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<MeetingLink>, AppError> {
    storage.get_meeting_links(&session_id)
}
