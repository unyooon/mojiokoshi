use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use screencapturekit::prelude::*;

use crate::audio::output_handler::AudioOutputHandler;
use crate::audio::{AudioBuffer, AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;

/// ScreenCaptureKit-based audio capture for macOS.
///
/// Captures system audio using the macOS ScreenCaptureKit framework.
/// Video is minimized (2x2) since only audio is needed.
pub struct ScreenCaptureKitCapture {
    state: AtomicU8,
    paused: Arc<AtomicBool>,
    config: Option<AudioConfig>,
    stream: Option<SCStream>,
}

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
            paused: Arc::new(AtomicBool::new(false)),
            config: None,
            stream: None,
        }
    }

    fn state_from_u8(val: u8) -> CaptureState {
        match val {
            STATE_CAPTURING => CaptureState::Capturing,
            STATE_PAUSED => CaptureState::Paused,
            _ => CaptureState::Idle,
        }
    }

    fn build_stream(&self, sender: Sender<AudioBuffer>) -> Result<SCStream, AppError> {
        let content = SCShareableContent::get()
            .map_err(|e| AppError::AudioCapture(format!("Failed to get content: {e}")))?;

        let display = content
            .displays()
            .into_iter()
            .next()
            .ok_or_else(|| AppError::AudioCapture("No display found".to_string()))?;

        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build();

        let stream_config = SCStreamConfiguration::new()
            .with_width(2)
            .with_height(2)
            .with_captures_audio(true)
            .with_sample_rate(16000)
            .with_channel_count(1)
            .with_excludes_current_process_audio(true);

        let mut stream = SCStream::new(&filter, &stream_config);

        let handler = AudioOutputHandler::new(sender, Arc::clone(&self.paused));
        stream.add_output_handler(handler, SCStreamOutputType::Audio);

        Ok(stream)
    }
}

impl AudioCapture for ScreenCaptureKitCapture {
    fn start(&mut self, config: &AudioConfig, sender: Sender<AudioBuffer>) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current == STATE_CAPTURING {
            return Err(AppError::AudioCapture("Already capturing".to_string()));
        }

        let stream = self.build_stream(sender)?;
        stream
            .start_capture()
            .map_err(|e| AppError::AudioCapture(format!("Failed to start capture: {e}")))?;

        self.config = Some(config.clone());
        self.paused.store(false, Ordering::Release);
        self.stream = Some(stream);
        self.state.store(STATE_CAPTURING, Ordering::Release);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AppError> {
        if let Some(ref stream) = self.stream {
            stream
                .stop_capture()
                .map_err(|e| AppError::AudioCapture(format!("Failed to stop capture: {e}")))?;
        }
        self.stream = None;
        self.config = None;
        self.paused.store(false, Ordering::Release);
        self.state.store(STATE_IDLE, Ordering::Release);
        Ok(())
    }

    fn pause(&mut self) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current != STATE_CAPTURING {
            return Err(AppError::AudioCapture(
                "Not currently capturing".to_string(),
            ));
        }
        self.paused.store(true, Ordering::Release);
        self.state.store(STATE_PAUSED, Ordering::Release);
        Ok(())
    }

    fn resume(&mut self) -> Result<(), AppError> {
        let current = self.state.load(Ordering::Acquire);
        if current != STATE_PAUSED {
            return Err(AppError::AudioCapture("Not currently paused".to_string()));
        }
        self.paused.store(false, Ordering::Release);
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
    fn test_pause_when_idle_errors() {
        let mut capture = ScreenCaptureKitCapture::new();
        assert!(capture.pause().is_err());
    }

    #[test]
    fn test_resume_when_idle_errors() {
        let mut capture = ScreenCaptureKitCapture::new();
        assert!(capture.resume().is_err());
    }

    #[test]
    fn test_default_creates_idle() {
        let capture = ScreenCaptureKitCapture::default();
        assert_eq!(capture.state(), CaptureState::Idle);
        assert!(capture.stream.is_none());
    }

    /// Integration test: starts a real SCStream for 2 seconds.
    /// Requires screen recording permission. Run manually:
    ///   cargo test test_real_stream -- --ignored
    #[test]
    #[ignore]
    fn test_real_stream_receives_audio() {
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();
        let (sender, receiver) = std::sync::mpsc::channel::<AudioBuffer>();

        capture.start(&config, sender).unwrap();
        assert_eq!(capture.state(), CaptureState::Capturing);

        // Wait up to 2 seconds for audio buffers
        let mut received = 0;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            if receiver
                .recv_timeout(std::time::Duration::from_millis(100))
                .is_ok()
            {
                received += 1;
                if received >= 5 {
                    break;
                }
            }
        }

        capture.stop().unwrap();
        assert_eq!(capture.state(), CaptureState::Idle);
        assert!(
            received > 0,
            "Expected to receive audio buffers from real stream"
        );
    }
}
