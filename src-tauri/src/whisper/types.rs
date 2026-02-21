use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_ms: f64,
    pub end_ms: f64,
    pub confidence: f32,
    pub is_partial: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEvent {
    pub id: String,
    pub timestamp: i64,
    pub text: String,
    pub start_ms: f64,
    pub end_ms: f64,
    pub confidence: f32,
    pub is_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WhisperConfig {
    pub model_path: String,
    pub language: String,
    pub translate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum WhisperModelStatus {
    NotDownloaded,
    Downloading { progress: f32 },
    Ready { path: String },
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            language: "ja".to_string(),
            translate: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whisper_config_default() {
        let config = WhisperConfig::default();
        assert!(config.model_path.is_empty());
        assert_eq!(config.language, "ja");
        assert!(!config.translate);
    }

    #[test]
    fn transcription_segment_serialization_roundtrip() {
        let segment = TranscriptionSegment {
            text: "Hello".to_string(),
            start_ms: 0.0,
            end_ms: 1000.0,
            confidence: 0.95,
            is_partial: false,
        };
        let json = serde_json::to_string(&segment).unwrap();
        let deserialized: TranscriptionSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "Hello");
        assert!((deserialized.confidence - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn transcript_event_serialization_roundtrip() {
        let event = TranscriptEvent {
            id: "test-id".to_string(),
            timestamp: 1700000000000,
            text: "Hello world".to_string(),
            start_ms: 0.0,
            end_ms: 1000.0,
            confidence: 0.95,
            is_partial: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"isPartial\""));
        assert!(json.contains("\"startMs\""));
        assert!(json.contains("\"endMs\""));
        assert!(!json.contains("\"is_partial\""));
        let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized["text"], "Hello world");
        assert_eq!(deserialized["isPartial"], false);
    }

    #[test]
    fn whisper_model_status_serialization_roundtrip() {
        let status = WhisperModelStatus::Ready {
            path: "/models/ggml-base.bin".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: WhisperModelStatus = serde_json::from_str(&json).unwrap();
        match deserialized {
            WhisperModelStatus::Ready { path } => {
                assert_eq!(path, "/models/ggml-base.bin");
            }
            _ => panic!("Expected Ready variant"),
        }
    }
}
