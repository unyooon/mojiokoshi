use serde::Serialize;
use specta::Type;

/// Application-wide error type.
#[derive(Debug, thiserror::Error, Type)]
pub enum AppError {
    #[error("Audio capture error: {0}")]
    AudioCapture(String),

    #[error("Speech recognition error: {0}")]
    SpeechRecognition(String),

    #[error("AI analysis error: {0}")]
    AiAnalysis(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
