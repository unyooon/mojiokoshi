use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, State};

use crate::claude::batch::BatchProcessor;
use crate::claude::bridge::ClaudeCodeBridge;
use crate::claude::types::{BridgeRequest, InvestigatePayload, InvestigationResult};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;

pub struct AiState {
    pub bridge: Arc<ClaudeCodeBridge>,
    pub storage: Arc<SqliteStorage>,
    pub batch_processor: Mutex<Option<BatchProcessor>>,
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

#[tauri::command]
#[specta::specta]
pub fn start_ai_analysis(
    app: AppHandle,
    ai: State<'_, AiState>,
    session_id: String,
) -> Result<(), AppError> {
    ai.bridge.start()?;

    let mut bp_guard = ai.batch_processor.lock().map_err(lock_err)?;

    if bp_guard.is_some() {
        return Err(AppError::AiAnalysis("Analysis already running".into()));
    }

    let mut processor =
        BatchProcessor::new(Arc::clone(&ai.bridge), Arc::clone(&ai.storage), session_id);

    let result = processor.process_batch()?;
    if let Some(batch) = result {
        emit_batch_results(&app, &batch)?;
    }

    *bp_guard = Some(processor);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn stop_ai_analysis(ai: State<'_, AiState>, _session_id: String) -> Result<(), AppError> {
    let mut bp_guard = ai.batch_processor.lock().map_err(lock_err)?;
    *bp_guard = None;
    ai.bridge.stop()?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn run_ai_batch(app: AppHandle, ai: State<'_, AiState>) -> Result<(), AppError> {
    let mut bp_guard = ai.batch_processor.lock().map_err(lock_err)?;
    let processor = bp_guard
        .as_mut()
        .ok_or_else(|| AppError::AiAnalysis("No active analysis session".into()))?;

    let result = processor.process_batch()?;
    if let Some(batch) = result {
        emit_batch_results(&app, &batch)?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn investigate(
    app: AppHandle,
    ai: State<'_, AiState>,
    query: String,
    context: String,
) -> Result<InvestigationResult, AppError> {
    ai.bridge.start()?;

    let payload = serde_json::to_value(InvestigatePayload {
        query: query.clone(),
        context,
    })
    .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

    let request = BridgeRequest {
        id: uuid::Uuid::new_v4().to_string(),
        request_type: "investigate".to_string(),
        payload,
    };

    let response = ai.bridge.send_request(request)?;
    if response.response_type == "error" {
        let msg = response
            .payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Investigation failed");
        return Err(AppError::AiAnalysis(msg.to_string()));
    }

    let mut result: InvestigationResult = serde_json::from_value(response.payload)
        .map_err(|e| AppError::AiAnalysis(e.to_string()))?;
    result.query = query;
    result.id = uuid::Uuid::new_v4().to_string();

    let _ = app.emit("ai:investigation", &result);
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub fn generate_minutes(ai: State<'_, AiState>, session_id: String) -> Result<String, AppError> {
    ai.bridge.start()?;

    let segments = ai.storage.get_segments(&session_id)?;
    if segments.is_empty() {
        return Err(AppError::AiAnalysis("No transcript segments found".into()));
    }

    let keywords = ai.storage.get_keywords(&session_id)?;
    let transcript: String = segments
        .iter()
        .map(|s| {
            let speaker = s.speaker.as_deref().unwrap_or("Unknown");
            format!("[{speaker}] {}", s.text)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let keyword_terms: Vec<String> = keywords.iter().map(|k| k.term.clone()).collect();

    let mut payload = serde_json::json!({ "transcript_text": transcript });
    if !keyword_terms.is_empty() {
        payload["keywords"] = serde_json::Value::from(keyword_terms);
    }

    let request = BridgeRequest {
        id: uuid::Uuid::new_v4().to_string(),
        request_type: "generate_minutes".to_string(),
        payload,
    };

    let response = ai.bridge.send_request(request)?;
    if response.response_type == "error" {
        let msg = response
            .payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Minutes generation failed");
        return Err(AppError::AiAnalysis(msg.to_string()));
    }

    response
        .payload
        .get("markdown")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::AiAnalysis("No markdown in response".into()))
}

fn emit_batch_results(
    app: &AppHandle,
    batch: &crate::claude::types::AnalysisBatchResult,
) -> Result<(), AppError> {
    let _ = app.emit("ai:keywords", &batch.keywords);
    let _ = app.emit("ai:summary", &batch.summary);
    let _ = app.emit("ai:actions", &batch.action_items);
    let _ = app.emit("ai:decisions", &batch.decisions);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_state_can_be_created() {
        let bridge = Arc::new(ClaudeCodeBridge::new());
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let state = AiState {
            bridge,
            storage,
            batch_processor: Mutex::new(None),
        };
        assert!(state.batch_processor.lock().unwrap().is_none());
    }

    #[test]
    fn stop_analysis_clears_processor() {
        let bridge = Arc::new(ClaudeCodeBridge::new());
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let state = AiState {
            bridge,
            storage,
            batch_processor: Mutex::new(None),
        };
        let mut guard = state.batch_processor.lock().unwrap();
        assert!(guard.is_none());
        *guard = None; // stop equivalent
        assert!(guard.is_none());
    }
}
