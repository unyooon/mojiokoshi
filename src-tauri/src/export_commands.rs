use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::export::markdown::{self, ExportOptions, ExportResult};
use crate::storage::sqlite::SqliteStorage;

/// Generate a markdown export for a session.
#[tauri::command]
#[specta::specta]
pub fn export_markdown(
    storage: State<'_, Arc<SqliteStorage>>,
    options: ExportOptions,
) -> Result<ExportResult, AppError> {
    markdown::generate_markdown(&storage, &options)
}

/// Save exported content to the user's Documents directory.
///
/// Returns the absolute path of the written file.
#[tauri::command]
#[specta::specta]
pub fn save_export_file(content: String, filename: String) -> Result<String, AppError> {
    let docs_dir = dirs::document_dir()
        .ok_or_else(|| AppError::Storage("Cannot determine Documents directory".into()))?;
    let path = docs_dir.join(&filename);
    std::fs::write(&path, &content).map_err(|e| AppError::Storage(e.to_string()))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Storage("Path contains invalid UTF-8".into()))
}
