use crate::error::AppError;
use super::{SpeechRecognizer, TranscriptionSegment, VoiceActivityDetector};

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
    fn is_speech(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<bool, AppError> {
        let prob = self.speech_probability(samples, sample_rate)?;
        Ok(prob > self.threshold)
    }

    fn speech_probability(
        &self,
        samples: &[f32],
        _sample_rate: u32,
    ) -> Result<f32, AppError> {
        if samples.is_empty() {
            return Ok(0.0);
        }
        // Simple energy-based stub: compute RMS energy
        let energy: f32 =
            samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        Ok(energy.sqrt().min(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_recognizer_returns_empty() {
        let recognizer = StubRecognizer;
        let result = recognizer.transcribe(&[0.0; 100], 16000).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn stub_vad_empty_samples() {
        let vad = StubVad::default();
        let prob = vad.speech_probability(&[], 16000).unwrap();
        assert!((prob - 0.0).abs() < f32::EPSILON);
        assert!(!vad.is_speech(&[], 16000).unwrap());
    }

    #[test]
    fn stub_vad_silence() {
        let vad = StubVad::default();
        let silence = vec![0.0_f32; 1600];
        assert!(!vad.is_speech(&silence, 16000).unwrap());
    }

    #[test]
    fn stub_vad_loud_signal() {
        let vad = StubVad::default();
        let loud = vec![0.9_f32; 1600];
        assert!(vad.is_speech(&loud, 16000).unwrap());
    }

    #[test]
    fn stub_vad_custom_threshold() {
        let vad = StubVad::new(0.1);
        // Low-energy signal that passes low threshold
        let samples = vec![0.2_f32; 1600];
        assert!(vad.is_speech(&samples, 16000).unwrap());
    }

    #[test]
    fn stub_vad_probability_clamped() {
        let vad = StubVad::default();
        let loud = vec![2.0_f32; 100];
        let prob = vad.speech_probability(&loud, 16000).unwrap();
        assert!(prob <= 1.0);
    }
}
