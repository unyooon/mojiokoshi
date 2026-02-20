pub mod model;
pub mod pipeline;
pub mod recognizer;
pub mod silero_vad;
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
    fn is_speech(&self, samples: &[f32], sample_rate: u32) -> Result<bool, AppError>;

    /// Return the probability that the audio contains speech.
    fn speech_probability(&self, samples: &[f32], sample_rate: u32) -> Result<f32, AppError>;
}

use std::path::Path;
use std::sync::Arc;

/// Pair of recognizer and VAD instances.
pub type ModelPair = (
    Arc<dyn SpeechRecognizer + Send + Sync>,
    Arc<dyn VoiceActivityDetector + Send + Sync>,
);

/// Load recognizer and VAD from model and VAD paths.
pub fn load_models(model_path: &str, vad_path: &Path) -> Result<ModelPair, AppError> {
    let config = WhisperConfig {
        model_path: model_path.to_string(),
        language: "ja".to_string(),
        translate: false,
    };
    let r = recognizer::WhisperRecognizer::new(&config)?;
    let v = silero_vad::SileroVad::new(vad_path, 0.5)?;
    Ok((Arc::new(r), Arc::new(v)))
}

#[cfg(test)]
pub mod stub {
    use super::{SpeechRecognizer, TranscriptionSegment, VoiceActivityDetector};
    use crate::error::AppError;

    pub struct StubRecognizer;

    impl SpeechRecognizer for StubRecognizer {
        fn transcribe(
            &self,
            _samples: &[f32],
            _sample_rate: u32,
        ) -> Result<Vec<TranscriptionSegment>, AppError> {
            Ok(vec![])
        }
    }

    pub struct StubVad {
        threshold: f32,
    }

    impl StubVad {
        pub fn new(threshold: f32) -> Self {
            Self { threshold }
        }
    }

    impl Default for StubVad {
        fn default() -> Self {
            Self::new(0.5)
        }
    }

    impl VoiceActivityDetector for StubVad {
        fn is_speech(&self, samples: &[f32], sample_rate: u32) -> Result<bool, AppError> {
            let prob = self.speech_probability(samples, sample_rate)?;
            Ok(prob > self.threshold)
        }

        fn speech_probability(&self, samples: &[f32], _sample_rate: u32) -> Result<f32, AppError> {
            if samples.is_empty() {
                return Ok(0.0);
            }
            let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
            Ok(energy.sqrt().min(1.0))
        }
    }
}
