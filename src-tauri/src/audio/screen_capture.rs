use std::sync::atomic::{AtomicU8, Ordering};

use crate::audio::{AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;

/// ScreenCaptureKit-based audio capture for macOS.
///
/// Captures system audio from the selected application using
/// macOS ScreenCaptureKit framework.
pub struct ScreenCaptureKitCapture {
    state: AtomicU8,
    config: Option<AudioConfig>,
}

// CaptureState encoding for AtomicU8
const STATE_IDLE: u8 = 0;
const STATE_CAPTURING: u8 = 1;
const STATE_PAUSED: u8 = 2;

impl Default for ScreenCaptureKitCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureKitCapture {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_IDLE),
            config: None,
        }
    }

    fn state_from_u8(val: u8) -> CaptureState {
        match val {
            STATE_CAPTURING => CaptureState::Capturing,
            STATE_PAUSED => CaptureState::Paused,
            _ => CaptureState::Idle,
        }
    }
}

impl AudioCapture for ScreenCaptureKitCapture {
    fn start(&mut self, config: &AudioConfig) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current == STATE_CAPTURING {
            return Err(AppError::AudioCapture(
                "Already capturing".to_string(),
            ));
        }
        self.config = Some(config.clone());
        self.state.store(STATE_CAPTURING, Ordering::Release);
        // TODO: Initialize SCStream with ScreenCaptureKit
        // This will be implemented when screencapturekit-rs is added
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AppError> {
        self.state.store(STATE_IDLE, Ordering::Release);
        self.config = None;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current != STATE_CAPTURING {
            return Err(AppError::AudioCapture(
                "Not currently capturing".to_string(),
            ));
        }
        self.state.store(STATE_PAUSED, Ordering::Release);
        Ok(())
    }

    fn resume(&mut self) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current != STATE_PAUSED {
            return Err(AppError::AudioCapture(
                "Not currently paused".to_string(),
            ));
        }
        self.state.store(STATE_CAPTURING, Ordering::Release);
        Ok(())
    }

    fn state(&self) -> CaptureState {
        Self::state_from_u8(self.state.load(Ordering::Acquire))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_idle() {
        let capture = ScreenCaptureKitCapture::new();
        assert_eq!(capture.state(), CaptureState::Idle);
    }

    #[test]
    fn test_start_stop_lifecycle() {
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();

        capture.start(&config).unwrap();
        assert_eq!(capture.state(), CaptureState::Capturing);

        capture.stop().unwrap();
        assert_eq!(capture.state(), CaptureState::Idle);
    }

    #[test]
    fn test_pause_resume() {
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();

        capture.start(&config).unwrap();
        capture.pause().unwrap();
        assert_eq!(capture.state(), CaptureState::Paused);

        capture.resume().unwrap();
        assert_eq!(capture.state(), CaptureState::Capturing);
    }

    #[test]
    fn test_double_start_errors() {
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();

        capture.start(&config).unwrap();
        assert!(capture.start(&config).is_err());
    }

    #[test]
    fn test_pause_when_idle_errors() {
        let mut capture = ScreenCaptureKitCapture::new();
        assert!(capture.pause().is_err());
    }
}
