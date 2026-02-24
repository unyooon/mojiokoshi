use std::path::Path;
use std::sync::Mutex;

use ndarray::Array3;
use ort::session::Session;
use ort::value::Tensor;

use super::VoiceActivityDetector;
use crate::error::AppError;

/// Expected chunk size for Silero VAD v5 at 16kHz.
const CHUNK_SIZE: usize = 512;

/// Combined LSTM state dimensions for Silero VAD v5: [2, 1, 128].
const STATE_SHAPE: [usize; 3] = [2, 1, 128];

pub struct SileroVad {
    session: Mutex<Session>,
    threshold: f32,
    state: Mutex<Array3<f32>>,
}

impl SileroVad {
    pub fn new(model_path: &Path, threshold: f32) -> Result<Self, AppError> {
        let session = Session::builder()
            .map_err(|e| AppError::Internal(format!("ORT session builder: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::Internal(format!("ORT model load: {e}")))?;

        let inputs: Vec<String> = session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let outputs: Vec<String> = session
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        log::info!("SileroVad model inputs: {inputs:?}");
        log::info!("SileroVad model outputs: {outputs:?}");

        Ok(Self {
            session: Mutex::new(session),
            threshold,
            state: Mutex::new(Array3::zeros(STATE_SHAPE)),
        })
    }

    /// Reset LSTM state to zeros.
    pub fn reset(&self) -> Result<(), AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| AppError::Internal(format!("VAD state lock: {e}")))?;
        *state = Array3::zeros(STATE_SHAPE);
        Ok(())
    }

    fn run_inference(&self, samples: &[f32]) -> Result<f32, AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| AppError::Internal(format!("VAD state lock: {e}")))?;

        let chunk_len = samples.len();
        let input = Tensor::from_array(([1usize, chunk_len], samples.to_vec()))
            .map_err(|e| AppError::Internal(format!("input tensor: {e}")))?;

        let sr = Tensor::from_array(([1usize], vec![16000_i64]))
            .map_err(|e| AppError::Internal(format!("sr tensor: {e}")))?;

        let state_tensor = Tensor::from_array(state.clone())
            .map_err(|e| AppError::Internal(format!("state tensor: {e}")))?;

        let mut session = self
            .session
            .lock()
            .map_err(|e| AppError::Internal(format!("session lock: {e}")))?;

        let outputs = session
            .run(ort::inputs! {
                "input" => input,
                "state" => state_tensor,
                "sr" => sr
            })
            .map_err(|e| AppError::Internal(format!("ORT run: {e}")))?;

        let (_, prob_data) = outputs["output"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::Internal(format!("extract output: {e}")))?;
        let probability = prob_data.first().copied().unwrap_or(0.0);

        let (_, new_state) = outputs["stateN"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::Internal(format!("extract stateN: {e}")))?;

        *state = Array3::from_shape_vec(STATE_SHAPE, new_state.to_vec())
            .map_err(|e| AppError::Internal(format!("reshape stateN: {e}")))?;

        Ok(probability)
    }
}

impl VoiceActivityDetector for SileroVad {
    fn is_speech(&self, samples: &[f32], sample_rate: u32) -> Result<bool, AppError> {
        Ok(self.speech_probability(samples, sample_rate)? >= self.threshold)
    }

    fn speech_probability(&self, samples: &[f32], _sample_rate: u32) -> Result<f32, AppError> {
        if samples.is_empty() {
            return Ok(0.0);
        }

        // Pad or truncate to expected chunk size
        let chunk = if samples.len() < CHUNK_SIZE {
            let mut padded = samples.to_vec();
            padded.resize(CHUNK_SIZE, 0.0);
            padded
        } else {
            samples[..CHUNK_SIZE].to_vec()
        };

        self.run_inference(&chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_initializes_to_zeros() {
        let state = Array3::<f32>::zeros(STATE_SHAPE);
        assert_eq!(state.shape(), &[2, 1, 128]);
        assert!(state.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn silero_vad_requires_valid_model_path() {
        let result = SileroVad::new(Path::new("/nonexistent/model.onnx"), 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn silero_vad_constants() {
        assert_eq!(CHUNK_SIZE, 512);
        assert_eq!(STATE_SHAPE, [2, 1, 128]);
    }
}
