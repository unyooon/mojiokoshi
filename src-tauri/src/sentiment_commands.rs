use std::sync::Arc;

use tauri::State;

use crate::ai_commands::AiState;
use crate::error::AppError;
use crate::storage::sentiment_store::SentimentEntry;
use crate::storage::sqlite::SqliteStorage;

/// Emotion labels cycled through in stub mode based on segment index.
const STUB_EMOTIONS: &[(&str, f64)] = &[
    ("neutral", 0.0),
    ("positive", 0.6),
    ("negative", -0.5),
    ("excited", 0.8),
    ("concerned", -0.3),
    ("confused", -0.1),
];

/// Analyse sentiment for every segment in a session.
///
/// In stub mode (no real AI backend), generates deterministic mock
/// sentiments cycling through emotion labels. Results are persisted
/// and returned.
#[tauri::command]
#[specta::specta]
pub fn analyze_sentiment(
    storage: State<'_, Arc<SqliteStorage>>,
    _ai_state: State<'_, AiState>,
    session_id: String,
) -> Result<Vec<SentimentEntry>, AppError> {
    let segments = storage.get_segments(&session_id)?;
    if segments.is_empty() {
        return Err(AppError::AiAnalysis("No transcript segments found".into()));
    }

    // Clear any previous sentiments for this session before re-analysis.
    storage.delete_sentiments(&session_id)?;

    let mut results = Vec::with_capacity(segments.len());
    for (idx, segment) in segments.iter().enumerate() {
        let (emotion, score) = STUB_EMOTIONS[idx % STUB_EMOTIONS.len()];
        let confidence = 0.75 + 0.05 * ((idx % 5) as f64);
        let entry = SentimentEntry {
            id: 0,
            session_id: session_id.clone(),
            segment_id: segment.id,
            score,
            emotion: emotion.to_string(),
            confidence,
            timestamp: segment.start_time,
            created_at: String::new(),
        };
        storage.insert_sentiment(&entry)?;
        results.push(entry);
    }

    // Re-read from DB so ids and created_at are populated.
    storage.get_sentiments(&session_id)
}

/// Retrieve previously computed sentiments for a session.
#[tauri::command]
#[specta::specta]
pub fn get_sentiments(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<SentimentEntry>, AppError> {
    storage.get_sentiments(&session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Segment;

    fn setup_session_with_segments(count: usize) -> (Arc<SqliteStorage>, String) {
        use crate::storage::SessionStorage;
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let sid = storage.create_session("Sentiment Test").unwrap();
        for i in 0..count {
            storage
                .insert_segment(&Segment {
                    id: 0,
                    session_id: sid.clone(),
                    speaker: Some("Alice".to_string()),
                    text: format!("Segment {i}"),
                    start_time: i as f64 * 1.0,
                    end_time: i as f64 * 1.0 + 0.5,
                    confidence: Some(0.9),
                    is_partial: false,
                })
                .unwrap();
        }
        (storage, sid)
    }

    #[test]
    fn stub_generates_correct_emotion_cycle() {
        let (storage, sid) = setup_session_with_segments(6);

        // Clear and insert stub sentiments directly (same logic as analyze_sentiment)
        let segments = storage.get_segments(&sid).unwrap();
        for (idx, segment) in segments.iter().enumerate() {
            let (emotion, score) = STUB_EMOTIONS[idx % STUB_EMOTIONS.len()];
            let entry = SentimentEntry {
                id: 0,
                session_id: sid.clone(),
                segment_id: segment.id,
                score,
                emotion: emotion.to_string(),
                confidence: 0.8,
                timestamp: segment.start_time,
                created_at: String::new(),
            };
            storage.insert_sentiment(&entry).unwrap();
        }

        let results = storage.get_sentiments(&sid).unwrap();
        assert_eq!(results.len(), 6);
        assert_eq!(results[0].emotion, "neutral");
        assert_eq!(results[1].emotion, "positive");
        assert_eq!(results[2].emotion, "negative");
        assert_eq!(results[3].emotion, "excited");
        assert_eq!(results[4].emotion, "concerned");
        assert_eq!(results[5].emotion, "confused");
    }

    #[test]
    fn stub_scores_match_emotions() {
        let (storage, sid) = setup_session_with_segments(3);
        let segments = storage.get_segments(&sid).unwrap();
        for (idx, segment) in segments.iter().enumerate() {
            let (emotion, score) = STUB_EMOTIONS[idx % STUB_EMOTIONS.len()];
            let entry = SentimentEntry {
                id: 0,
                session_id: sid.clone(),
                segment_id: segment.id,
                score,
                emotion: emotion.to_string(),
                confidence: 0.8,
                timestamp: segment.start_time,
                created_at: String::new(),
            };
            storage.insert_sentiment(&entry).unwrap();
        }

        let results = storage.get_sentiments(&sid).unwrap();
        assert!((results[0].score - 0.0).abs() < f64::EPSILON);
        assert!((results[1].score - 0.6).abs() < f64::EPSILON);
        assert!((results[2].score - (-0.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn get_sentiments_returns_empty_for_no_analysis() {
        let (storage, sid) = setup_session_with_segments(2);
        let results = storage.get_sentiments(&sid).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn re_analysis_clears_previous_sentiments() {
        let (storage, sid) = setup_session_with_segments(2);
        let segments = storage.get_segments(&sid).unwrap();

        // First analysis
        for segment in &segments {
            let entry = SentimentEntry {
                id: 0,
                session_id: sid.clone(),
                segment_id: segment.id,
                score: 0.5,
                emotion: "positive".to_string(),
                confidence: 0.8,
                timestamp: segment.start_time,
                created_at: String::new(),
            };
            storage.insert_sentiment(&entry).unwrap();
        }
        assert_eq!(storage.get_sentiments(&sid).unwrap().len(), 2);

        // Simulate re-analysis: delete then re-insert
        storage.delete_sentiments(&sid).unwrap();
        for segment in &segments {
            let entry = SentimentEntry {
                id: 0,
                session_id: sid.clone(),
                segment_id: segment.id,
                score: -0.3,
                emotion: "concerned".to_string(),
                confidence: 0.9,
                timestamp: segment.start_time,
                created_at: String::new(),
            };
            storage.insert_sentiment(&entry).unwrap();
        }
        let results = storage.get_sentiments(&sid).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].emotion, "concerned");
    }
}
