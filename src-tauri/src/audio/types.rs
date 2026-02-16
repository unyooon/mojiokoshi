use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub sample_format: AudioSampleFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum AudioSampleFormat {
    F32,
    I16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub timestamp_ms: f64,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channel_count: 1,
            sample_format: AudioSampleFormat::F32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum CaptureState {
    Idle,
    Capturing,
    Paused,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channel_count, 1);
        assert!(matches!(config.sample_format, AudioSampleFormat::F32));
    }

    #[test]
    fn audio_config_serialization_roundtrip() {
        let config = AudioConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AudioConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sample_rate, config.sample_rate);
        assert_eq!(deserialized.channel_count, config.channel_count);
    }

    #[test]
    fn capture_state_serialization_roundtrip() {
        for state in [CaptureState::Idle, CaptureState::Capturing, CaptureState::Paused] {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: CaptureState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn audio_buffer_serialization_roundtrip() {
        let buffer = AudioBuffer {
            samples: vec![0.0, 0.5, -0.5, 1.0],
            sample_rate: 16000,
            timestamp_ms: 1234.5,
        };
        let json = serde_json::to_string(&buffer).unwrap();
        let deserialized: AudioBuffer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.samples, buffer.samples);
        assert_eq!(deserialized.sample_rate, buffer.sample_rate);
        assert!((deserialized.timestamp_ms - buffer.timestamp_ms).abs() < f64::EPSILON);
    }
}
