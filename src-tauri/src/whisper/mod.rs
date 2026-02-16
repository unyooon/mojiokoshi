/// Speech recognition trait for converting audio to text.
///
/// Implementations should handle model loading and inference
/// (e.g., whisper-rs with large-v3-turbo model).
pub trait SpeechRecognizer: Send + Sync {
    /// Transcribe audio samples to text.
    fn transcribe(&self, samples: &[f32]) -> Result<String, crate::error::AppError>;
}

/// Voice activity detection trait.
///
/// Implementations detect speech segments in audio streams
/// (e.g., Silero VAD).
pub trait VoiceActivityDetector: Send + Sync {
    /// Detect if the given audio samples contain speech.
    fn is_speech(&self, samples: &[f32]) -> Result<bool, crate::error::AppError>;
}
