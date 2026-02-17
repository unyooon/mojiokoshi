use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::storage::keyword_dictionary_store::DictionaryKeyword;
use crate::storage::sqlite::SqliteStorage;

#[tauri::command]
#[specta::specta]
pub fn add_dictionary_keyword(
    storage: State<'_, Arc<SqliteStorage>>,
    term: String,
    reading: Option<String>,
    definition: Option<String>,
    category: String,
) -> Result<i64, AppError> {
    storage.add_dictionary_keyword(&term, reading.as_deref(), definition.as_deref(), &category)
}

#[tauri::command]
#[specta::specta]
pub fn update_dictionary_keyword(
    storage: State<'_, Arc<SqliteStorage>>,
    id: i64,
    term: String,
    reading: Option<String>,
    definition: Option<String>,
    category: String,
) -> Result<(), AppError> {
    storage.update_dictionary_keyword(
        id,
        &term,
        reading.as_deref(),
        definition.as_deref(),
        &category,
    )
}

#[tauri::command]
#[specta::specta]
pub fn delete_dictionary_keyword(
    storage: State<'_, Arc<SqliteStorage>>,
    id: i64,
) -> Result<(), AppError> {
    storage.delete_dictionary_keyword(id)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_dictionary_keywords(
    storage: State<'_, Arc<SqliteStorage>>,
) -> Result<Vec<DictionaryKeyword>, AppError> {
    storage.get_all_dictionary_keywords()
}
