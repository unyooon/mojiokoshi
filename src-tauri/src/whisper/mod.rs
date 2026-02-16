pub mod pipeline;
pub mod stub;
pub mod types;

pub use types::*;

use crate::error::AppError;

/// Speech recognition trait for converting audio to text.
///
/// Implementations should handle model loading and inference
/// (e.g., whisper-rs with large-v3-turbo model).
pub trait SpeechRecognizer: Send + Sync {
    /// Transcribe audio samples to text segments.
    fn transcribe(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<TranscriptionSegment>, AppError>;
}

/// Voice activity detection trait.
///
/// Implementations detect speech segments in audio streams
/// (e.g., Silero VAD).
pub trait VoiceActivityDetector: Send + Sync {
    /// Detect if the given audio samples contain speech.
    fn is_speech(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<bool, AppError>;

    /// Return the probability that the audio contains speech.
    fn speech_probability(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<f32, AppError>;
}
