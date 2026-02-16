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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WhisperConfig {
    pub model_path: String,
    pub language: String,
    pub translate: bool,
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
        let deserialized: TranscriptionSegment =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "Hello");
        assert!((deserialized.confidence - 0.95).abs() < f32::EPSILON);
    }
}
