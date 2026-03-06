use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::export::markdown::{self, ExportOptions, ExportResult};
use crate::export::pdf;
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

/// Generate a PDF export for a session.
///
/// Fetches session data via `export_markdown` (full transcript with all sections),
/// then converts the generated Markdown to a PDF document.
///
/// # Parameters
///
/// * `storage` - Injected SQLite storage state.
/// * `session_id` - The session to export.
///
/// # Returns
///
/// Raw PDF bytes ready to be saved or transferred.
///
/// # Errors
///
/// Returns `AppError` when session data cannot be fetched or PDF generation fails.
#[tauri::command]
#[specta::specta]
pub fn export_pdf(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<u8>, AppError> {
    let options = ExportOptions {
        session_id: session_id.clone(),
        include_summary: true,
        include_actions: true,
        include_keywords: true,
        include_transcript: true,
    };
    let result = markdown::generate_markdown(&storage, &options)?;
    pdf::generate_pdf(&result.filename, &result.content)
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
