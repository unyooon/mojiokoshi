use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, State};

use crate::claude::batch::BatchProcessor;
use crate::claude::bridge::ClaudeCodeBridge;
use crate::claude::types::{
    BridgeRequest, FormattedTranscript, InvestigatePayload, InvestigationResult,
    QuestionSuggestion, TranscriptSegmentForAi,
};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;

/// State for the AI analysis pipeline.
pub struct AiState {
    /// Bridge to the Claude Code sidecar process.
    pub bridge: Arc<ClaudeCodeBridge>,
    /// Persistent storage for segments and keywords.
    pub storage: Arc<SqliteStorage>,
    /// The active batch processor, if analysis is running.
    pub batch_processor: Mutex<Option<BatchProcessor>>,
    /// End timestamp (ms) of the last successfully formatted transcript segment.
    pub last_format_end_ms: Mutex<f64>,
    /// The most recently formatted transcript text for continuity.
    pub last_formatted_text: Mutex<Option<String>>,
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

/// Format the transcript segments using the AI sidecar, incrementally from the last formatted position.
///
/// # Errors
///
/// Returns `AppError::AiAnalysis` if the bridge request fails or no new segments are available.
#[tauri::command]
#[specta::specta]
pub fn format_transcript(
    app: AppHandle,
    ai: State<'_, AiState>,
    session_id: String,
) -> Result<FormattedTranscript, AppError> {
    ai.bridge.start()?;

    let last_end_ms = *ai.last_format_end_ms.lock().map_err(lock_err)?;
    let last_formatted = ai.last_formatted_text.lock().map_err(lock_err)?.clone();

    let all_segments = ai.storage.get_segments(&session_id)?;
    let new_segments: Vec<TranscriptSegmentForAi> = all_segments
        .iter()
        .filter(|s| s.start_time >= last_end_ms && !s.is_partial)
        .map(|s| TranscriptSegmentForAi {
            speaker: s.speaker.clone(),
            text: s.text.clone(),
            start_time: s.start_time,
            end_time: s.end_time,
        })
        .collect();

    if new_segments.is_empty() {
        return Err(AppError::AiAnalysis("No new segments to format".into()));
    }

    let payload = serde_json::to_value(crate::claude::types::FormatTranscriptPayload {
        segments: new_segments,
        previous_formatted: last_formatted,
    })
    .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

    let request = BridgeRequest {
        id: uuid::Uuid::new_v4().to_string(),
        request_type: "format_transcript".to_string(),
        payload,
    };

    let response = ai.bridge.send_request(request)?;
    if response.response_type == "error" {
        let msg = response
            .payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Transcript formatting failed");
        return Err(AppError::AiAnalysis(msg.to_string()));
    }

    let result: FormattedTranscript = serde_json::from_value(response.payload)
        .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

    *ai.last_format_end_ms.lock().map_err(lock_err)? = result.last_segment_end_ms;
    *ai.last_formatted_text.lock().map_err(lock_err)? = Some(result.formatted_text.clone());

    let _ = app.emit("ai:formatted-transcript", &result);
    Ok(result)
}

/// Suggest follow-up questions based on the current transcript and summary.
///
/// # Errors
///
/// Returns `AppError::AiAnalysis` if the bridge request fails or the response is malformed.
#[tauri::command]
#[specta::specta]
pub fn suggest_questions(
    ai: State<'_, AiState>,
    session_id: String,
) -> Result<Vec<QuestionSuggestion>, AppError> {
    ai.bridge.start()?;

    let segments = ai.storage.get_segments(&session_id)?;
    let recent_segments: Vec<&crate::storage::Segment> = segments
        .iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let transcript: String = recent_segments
        .iter()
        .map(|s| {
            let speaker = s.speaker.as_deref().unwrap_or("Unknown");
            format!("[{speaker}] {}", s.text)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let payload = serde_json::json!({
        "transcript_text": transcript,
        "session_id": session_id,
    });

    let request = BridgeRequest {
        id: uuid::Uuid::new_v4().to_string(),
        request_type: "suggest_questions".to_string(),
        payload,
    };

    let response = ai.bridge.send_request(request)?;
    if response.response_type == "error" {
        let msg = response
            .payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Question suggestion failed");
        return Err(AppError::AiAnalysis(msg.to_string()));
    }

    let suggestions: Vec<QuestionSuggestion> = serde_json::from_value(response.payload)
        .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

    Ok(suggestions)
}

fn emit_batch_results(
    app: &AppHandle,
    batch: &crate::claude::types::AnalysisBatchResult,
) -> Result<(), AppError> {
    let _ = app.emit("ai:keywords", &batch.keywords);
    let _ = app.emit("ai:summary", &batch.summary);
    let _ = app.emit("ai:actions", &batch.action_items);
    let _ = app.emit("ai:decisions", &batch.decisions);
    let _ = app.emit("ai:topics", &batch.topics);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ai_state() -> AiState {
        let bridge = Arc::new(ClaudeCodeBridge::new());
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        AiState {
            bridge,
            storage,
            batch_processor: Mutex::new(None),
            last_format_end_ms: Mutex::new(0.0),
            last_formatted_text: Mutex::new(None),
        }
    }

    #[test]
    fn ai_state_can_be_created() {
        let state = make_ai_state();
        assert!(state.batch_processor.lock().unwrap().is_none());
        assert!(*state.last_format_end_ms.lock().unwrap() == 0.0);
        assert!(state.last_formatted_text.lock().unwrap().is_none());
    }

    #[test]
    fn stop_analysis_clears_processor() {
        let state = make_ai_state();
        let mut guard = state.batch_processor.lock().unwrap();
        assert!(guard.is_none());
        *guard = None; // stop equivalent
        assert!(guard.is_none());
    }
}
