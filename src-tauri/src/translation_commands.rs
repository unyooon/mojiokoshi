use std::sync::Arc;

use tauri::State;

use crate::ai_commands::AiState;
use crate::claude::types::BridgeRequest;
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::translation_store::TranslationEntry;

/// Translate all segments for a session into the specified target language.
///
/// For each segment the function builds a prompt, sends it through the AI
/// bridge, persists the result, and returns the full list of translations.
/// When the bridge runs in stub mode the translated text is a simple
/// `[Translation] <original>` prefix so that the feature can be exercised
/// without an API key.
#[tauri::command]
#[specta::specta]
pub async fn translate_segments(
    storage: State<'_, Arc<SqliteStorage>>,
    ai_state: State<'_, AiState>,
    session_id: String,
    target_lang: String,
) -> Result<Vec<TranslationEntry>, AppError> {
    let segments = storage.get_segments(&session_id)?;
    if segments.is_empty() {
        return Err(AppError::AiAnalysis(
            "No transcript segments to translate".into(),
        ));
    }

    // Detect source language (opposite of target).
    let source_lang = if target_lang == "ja" { "en" } else { "ja" };

    ai_state.bridge.start()?;

    for segment in &segments {
        let payload = serde_json::json!({
            "source_lang": source_lang,
            "target_lang": target_lang,
            "text": segment.text,
        });

        let request = BridgeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "translate".to_string(),
            payload,
        };

        let response = ai_state.bridge.send_request(request)?;

        let translated_text = if response.response_type == "error" {
            let msg = response
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Translation failed");
            return Err(AppError::AiAnalysis(msg.to_string()));
        } else {
            response
                .payload
                .get("translated_text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        storage.insert_translation(
            &session_id,
            segment.id,
            source_lang,
            &target_lang,
            &segment.text,
            &translated_text,
        )?;
    }

    storage.get_translations(&session_id, &target_lang)
}

/// Retrieve previously stored translations for a session.
#[tauri::command]
#[specta::specta]
pub fn get_translations(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
    target_lang: String,
) -> Result<Vec<TranslationEntry>, AppError> {
    storage.get_translations(&session_id, &target_lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_entry_is_serializable() {
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
        assert!(json.contains("\"translated_text\":\"Test\""));
    }
}
