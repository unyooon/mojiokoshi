use log::{debug, info};
use std::sync::Mutex;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::{SpeechRecognizer, TranscriptionSegment, WhisperConfig};
use crate::error::AppError;

fn whisper_err(e: whisper_rs::WhisperError) -> AppError {
    AppError::SpeechRecognition(e.to_string())
}

pub struct WhisperRecognizer {
    ctx: Mutex<WhisperContext>,
    language: String,
    translate: bool,
}

impl WhisperRecognizer {
    pub fn new(config: &WhisperConfig) -> Result<Self, AppError> {
        let params = WhisperContextParameters::default();
        let ctx =
            WhisperContext::new_with_params(&config.model_path, params).map_err(whisper_err)?;
        info!(
            "WhisperRecognizer initialized (model={}, lang={})",
            config.model_path, config.language
        );
        Ok(Self {
            ctx: Mutex::new(ctx),
            language: config.language.clone(),
            translate: config.translate,
        })
    }
}

impl SpeechRecognizer for WhisperRecognizer {
    fn transcribe(
        &self,
        samples: &[f32],
        _sample_rate: u32,
    ) -> Result<Vec<TranscriptionSegment>, AppError> {
        debug!(
            "Transcribing {} samples ({:.1}s of audio)",
            samples.len(),
            samples.len() as f32 / 16000.0
        );
        let ctx = self
            .ctx
            .lock()
            .map_err(|e| AppError::SpeechRecognition(e.to_string()))?;
        let mut state = ctx.create_state().map_err(whisper_err)?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(&self.language));
        params.set_translate(self.translate);

        state.full(params, samples).map_err(whisper_err)?;

        let n = state.full_n_segments().map_err(whisper_err)?;

        let mut segments = Vec::new();
        for i in 0..n {
            let text = state.full_get_segment_text_lossy(i).map_err(whisper_err)?;
            let t0 = state.full_get_segment_t0(i).map_err(whisper_err)?;
            let t1 = state.full_get_segment_t1(i).map_err(whisper_err)?;

            segments.push(TranscriptionSegment {
                text,
                start_ms: (t0 * 10) as f64,
                end_ms: (t1 * 10) as f64,
                confidence: 1.0,
                is_partial: false,
            });
        }
        Ok(segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizer_requires_valid_model_path() {
        let config = WhisperConfig {
            model_path: "/nonexistent/model.bin".to_string(),
            language: "en".to_string(),
            translate: false,
        };
        let result = WhisperRecognizer::new(&config);
        assert!(result.is_err());
    }
}
