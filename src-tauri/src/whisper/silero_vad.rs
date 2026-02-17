use std::path::Path;
use std::sync::Mutex;

use ndarray::Array3;
use ort::session::Session;
use ort::value::Tensor;

use super::VoiceActivityDetector;
use crate::error::AppError;

/// Expected chunk size for Silero VAD v5 at 16kHz.
const CHUNK_SIZE: usize = 512;

/// LSTM hidden/cell state dimensions: [num_layers=2, batch=1, hidden=64].
const STATE_SHAPE: [usize; 3] = [2, 1, 64];

struct VadState {
    h: Array3<f32>,
    c: Array3<f32>,
}

impl VadState {
    fn new() -> Self {
        Self {
            h: Array3::zeros(STATE_SHAPE),
            c: Array3::zeros(STATE_SHAPE),
        }
    }
}

pub struct SileroVad {
    session: Mutex<Session>,
    threshold: f32,
    state: Mutex<VadState>,
}

impl SileroVad {
    pub fn new(model_path: &Path, threshold: f32) -> Result<Self, AppError> {
        let session = Session::builder()
            .map_err(|e| AppError::Internal(format!("ORT session builder: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::Internal(format!("ORT model load: {e}")))?;

        Ok(Self {
            session: Mutex::new(session),
            threshold,
            state: Mutex::new(VadState::new()),
        })
    }

    /// Reset LSTM hidden/cell states to zeros.
    pub fn reset(&self) -> Result<(), AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| AppError::Internal(format!("VAD state lock: {e}")))?;
        *state = VadState::new();
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

        let h = Tensor::from_array(state.h.clone())
            .map_err(|e| AppError::Internal(format!("h tensor: {e}")))?;

        let c = Tensor::from_array(state.c.clone())
            .map_err(|e| AppError::Internal(format!("c tensor: {e}")))?;

        let mut session = self
            .session
            .lock()
            .map_err(|e| AppError::Internal(format!("session lock: {e}")))?;

        let outputs = session
            .run(ort::inputs! {
                "input" => input,
                "sr" => sr,
                "h" => h,
                "c" => c
            })
            .map_err(|e| AppError::Internal(format!("ORT run: {e}")))?;

        let (_, prob_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::Internal(format!("extract prob: {e}")))?;
        let probability = prob_data.first().copied().unwrap_or(0.0);

        let (_, hn_data) = outputs[1]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::Internal(format!("extract hn: {e}")))?;
        let (_, cn_data) = outputs[2]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::Internal(format!("extract cn: {e}")))?;

        state.h = Array3::from_shape_vec(STATE_SHAPE, hn_data.to_vec())
            .map_err(|e| AppError::Internal(format!("reshape hn: {e}")))?;
        state.c = Array3::from_shape_vec(STATE_SHAPE, cn_data.to_vec())
            .map_err(|e| AppError::Internal(format!("reshape cn: {e}")))?;

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
    fn vad_state_initializes_to_zeros() {
        let state = VadState::new();
        assert_eq!(state.h.shape(), &[2, 1, 64]);
        assert_eq!(state.c.shape(), &[2, 1, 64]);
        assert!(state.h.iter().all(|&v| v == 0.0));
        assert!(state.c.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn silero_vad_requires_valid_model_path() {
        let result = SileroVad::new(Path::new("/nonexistent/model.onnx"), 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn silero_vad_constants() {
        assert_eq!(CHUNK_SIZE, 512);
        assert_eq!(STATE_SHAPE, [2, 1, 64]);
    }
}
