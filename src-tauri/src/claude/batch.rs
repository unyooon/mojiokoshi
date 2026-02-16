use std::sync::Arc;
use std::time::Duration;

use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::Segment;

use super::bridge::ClaudeCodeBridge;
use super::types::{AnalysisBatchPayload, AnalysisBatchResult, BridgeRequest};

/// Default batch interval: 3 minutes.
const DEFAULT_INTERVAL_SECS: u64 = 180;
/// Rolling context window: 15 minutes of full transcript.
const CONTEXT_WINDOW_MS: f64 = 15.0 * 60.0 * 1000.0;

/// Collects transcript segments and sends periodic batch analysis requests.
pub struct BatchProcessor {
    bridge: Arc<ClaudeCodeBridge>,
    storage: Arc<SqliteStorage>,
    interval: Duration,
    last_batch_end_ms: f64,
    session_id: String,
    meeting_topic: Option<String>,
}

impl BatchProcessor {
    pub fn new(
        bridge: Arc<ClaudeCodeBridge>,
        storage: Arc<SqliteStorage>,
        session_id: String,
    ) -> Self {
        Self {
            bridge,
            storage,
            interval: Duration::from_secs(DEFAULT_INTERVAL_SECS),
            last_batch_end_ms: 0.0,
            session_id,
            meeting_topic: None,
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_topic(mut self, topic: String) -> Self {
        self.meeting_topic = Some(topic);
        self
    }

    /// Returns the configured batch interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Run one batch cycle: collect new segments, analyze, persist keywords.
    pub fn process_batch(&mut self) -> Result<Option<AnalysisBatchResult>, AppError> {
        let segments = self.storage.get_segments(&self.session_id)?;
        let new_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| s.start_time >= self.last_batch_end_ms && !s.is_partial)
            .collect();

        if new_segments.is_empty() {
            return Ok(None);
        }

        let (transcript, from_ms, to_ms) = build_transcript(&new_segments);
        let context_summary = self.build_context_summary(&segments, from_ms);
        let existing = self.storage.get_keyword_terms(&self.session_id)?;

        let mut payload = serde_json::to_value(AnalysisBatchPayload {
            transcript_text: transcript,
            meeting_topic: self.meeting_topic.clone(),
            existing_keywords: existing,
        })
        .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

        if let Some(ctx) = context_summary {
            payload["context_summary"] = serde_json::Value::String(ctx);
        }
        payload["from_ms"] = serde_json::Value::from(from_ms);
        payload["to_ms"] = serde_json::Value::from(to_ms);

        let request = BridgeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "analyze_batch".to_string(),
            payload,
        };

        let response = self.bridge.send_request(request)?;

        if response.response_type == "error" {
            let msg = response
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown bridge error");
            return Err(AppError::AiAnalysis(msg.to_string()));
        }

        let result: AnalysisBatchResult = serde_json::from_value(response.payload)
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

        self.persist_keywords(&result)?;
        self.last_batch_end_ms = to_ms;

        Ok(Some(result))
    }

    /// Build a compressed context summary from older segments.
    fn build_context_summary(&self, all_segments: &[Segment], current_from: f64) -> Option<String> {
        let window_start = current_from - CONTEXT_WINDOW_MS;
        let older: Vec<&Segment> = all_segments
            .iter()
            .filter(|s| s.end_time < current_from && s.start_time >= window_start && !s.is_partial)
            .collect();

        if older.is_empty() {
            return None;
        }

        let text: String = older
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Truncate to ~2000 bytes for context compression (UTF-8 safe)
        let truncated = if text.len() > 2000 {
            format!("{}...", truncate_utf8(&text, 2000))
        } else {
            text
        };
        Some(truncated)
    }

    /// Persist new keywords and increment existing ones.
    fn persist_keywords(&self, result: &AnalysisBatchResult) -> Result<(), AppError> {
        for kw in &result.keywords {
            let existing = self
                .storage
                .get_keyword_terms(&self.session_id)?
                .iter()
                .any(|t| t.eq_ignore_ascii_case(&kw.term));

            if existing {
                self.storage
                    .increment_keyword_occurrence(&self.session_id, &kw.term)?;
            } else {
                self.storage.insert_keyword(&self.session_id, kw)?;
            }
        }
        Ok(())
    }
}

/// Truncate a UTF-8 string to at most `max_bytes` bytes without splitting
/// a multi-byte character.
fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Concatenate segment texts into a single transcript string.
fn build_transcript(segments: &[&Segment]) -> (String, f64, f64) {
    let mut from_ms = f64::MAX;
    let mut to_ms = f64::MIN;
    let mut parts = Vec::with_capacity(segments.len());

    for seg in segments {
        if seg.start_time < from_ms {
            from_ms = seg.start_time;
        }
        if seg.end_time > to_ms {
            to_ms = seg.end_time;
        }
        let speaker = seg.speaker.as_deref().unwrap_or("Unknown");
        parts.push(format!("[{speaker}] {}", seg.text));
    }

    if from_ms == f64::MAX {
        from_ms = 0.0;
    }
    if to_ms == f64::MIN {
        to_ms = 0.0;
    }

    (parts.join("\n"), from_ms, to_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn make_stub_bridge() -> Arc<ClaudeCodeBridge> {
        Arc::new(ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        })
    }

    fn make_storage_with_segments() -> (Arc<SqliteStorage>, String) {
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let session_id = storage.create_session("Test").unwrap();
        for (i, (start, end)) in [(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)].iter().enumerate() {
            storage
                .insert_segment(&Segment {
                    id: 0,
                    session_id: session_id.clone(),
                    speaker: Some("Alice".to_string()),
                    text: format!("Segment {i} about Rust and WebRTC"),
                    start_time: *start,
                    end_time: *end,
                    confidence: Some(0.9),
                    is_partial: false,
                })
                .unwrap();
        }
        (storage, session_id)
    }

    use crate::storage::SessionStorage;

    #[test]
    fn new_creates_processor() {
        let bridge = make_stub_bridge();
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let sid = storage.create_session("Test").unwrap();
        let bp = BatchProcessor::new(bridge, storage, sid);
        assert_eq!(bp.interval(), Duration::from_secs(180));
    }

    #[test]
    fn with_interval_sets_duration() {
        let bridge = make_stub_bridge();
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let sid = storage.create_session("Test").unwrap();
        let bp = BatchProcessor::new(bridge, storage, sid).with_interval(Duration::from_secs(60));
        assert_eq!(bp.interval(), Duration::from_secs(60));
    }

    #[test]
    fn process_batch_empty_returns_none() {
        let bridge = make_stub_bridge();
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let sid = storage.create_session("Empty").unwrap();
        let mut bp = BatchProcessor::new(bridge, storage, sid);
        let result = bp.process_batch().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn process_batch_with_segments_returns_result() {
        let (storage, session_id) = make_storage_with_segments();
        let bridge = make_stub_bridge();
        let mut bp = BatchProcessor::new(bridge, storage, session_id);
        let result = bp.process_batch().unwrap();
        assert!(result.is_some());
        let batch = result.unwrap();
        assert_eq!(batch.summary.text, "Stub summary");
    }

    #[test]
    fn process_batch_advances_cursor() {
        let (storage, session_id) = make_storage_with_segments();
        let bridge = make_stub_bridge();
        let mut bp = BatchProcessor::new(bridge, storage, session_id);
        let _ = bp.process_batch().unwrap();
        assert!(bp.last_batch_end_ms > 0.0);
        // Second batch with no new segments returns None
        let result = bp.process_batch().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn build_transcript_formats_segments() {
        let seg = Segment {
            id: 1,
            session_id: "s1".to_string(),
            speaker: Some("Bob".to_string()),
            text: "Hello world".to_string(),
            start_time: 100.0,
            end_time: 200.0,
            confidence: None,
            is_partial: false,
        };
        let (text, from, to) = build_transcript(&[&seg]);
        assert_eq!(text, "[Bob] Hello world");
        assert!((from - 100.0).abs() < f64::EPSILON);
        assert!((to - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_transcript_handles_no_speaker() {
        let seg = Segment {
            id: 1,
            session_id: "s1".to_string(),
            speaker: None,
            text: "No speaker".to_string(),
            start_time: 0.0,
            end_time: 1.0,
            confidence: None,
            is_partial: false,
        };
        let (text, _, _) = build_transcript(&[&seg]);
        assert_eq!(text, "[Unknown] No speaker");
    }
}
