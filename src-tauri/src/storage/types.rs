use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub target_app: Option<String>,
    pub whisper_model: Option<String>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SessionStatus {
    Active,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Segment {
    pub id: i64,
    pub session_id: String,
    pub speaker: Option<String>,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: Option<f64>,
    pub is_partial: bool,
}

/// A user-created bookmark that marks a specific point in a session's transcript.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Bookmark {
    /// Unique identifier for the bookmark (UUID v4).
    pub id: String,
    /// The session this bookmark belongs to.
    pub session_id: String,
    /// Optional reference to a specific transcript segment.
    pub segment_id: Option<String>,
    /// Optional freeform note attached to the bookmark.
    pub note: Option<String>,
    /// ISO 8601 timestamp of when the bookmark was created.
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_serialization_roundtrip() {
        let session = Session {
            id: "test-id".to_string(),
            title: "Test Session".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: None,
            target_app: None,
            whisper_model: None,
            status: SessionStatus::Active,
        };
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-id");
    }

    #[test]
    fn segment_serialization_roundtrip() {
        let segment = Segment {
            id: 1,
            session_id: "s1".to_string(),
            speaker: Some("Speaker A".to_string()),
            text: "Hello world".to_string(),
            start_time: 0.0,
            end_time: 1.5,
            confidence: Some(0.95),
            is_partial: false,
        };
        let json = serde_json::to_string(&segment).unwrap();
        let deserialized: Segment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "Hello world");
    }
}
