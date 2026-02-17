//! Test utilities and mock helpers for the MojiOkoshi backend.
//!
//! This module provides common test setup functions and mock
//! implementations using the mockall crate.

#[cfg(test)]
pub mod mocks {
    use crate::audio::{AudioBuffer, AudioCapture, AudioConfig, CaptureState};
    use crate::error::AppError;
    use mockall::mock;

    mock! {
        pub AudioCaptureImpl {}
        impl AudioCapture for AudioCaptureImpl {
            fn start(&mut self, config: &AudioConfig, sender: std::sync::mpsc::Sender<AudioBuffer>) -> Result<(), AppError>;
            fn stop(&mut self) -> Result<(), AppError>;
            fn pause(&mut self) -> Result<(), AppError>;
            fn resume(&mut self) -> Result<(), AppError>;
            fn state(&self) -> CaptureState;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mocks::MockAudioCaptureImpl;
    use crate::audio::{AudioBuffer, AudioCapture, AudioConfig, CaptureState};
    use crate::error::AppError;

    fn dummy_sender() -> std::sync::mpsc::Sender<AudioBuffer> {
        let (sender, _receiver) = std::sync::mpsc::channel();
        sender
    }

    #[test]
    fn mock_audio_capture_start_stop() {
        let mut mock = MockAudioCaptureImpl::new();
        mock.expect_start().returning(|_, _| Ok(()));
        mock.expect_state().returning(|| CaptureState::Capturing);
        mock.expect_stop().returning(|| Ok(()));

        let config = AudioConfig::default();
        assert!(mock.start(&config, dummy_sender()).is_ok());
        assert_eq!(mock.state(), CaptureState::Capturing);
        assert!(mock.stop().is_ok());
    }

    #[test]
    fn mock_audio_capture_error() {
        let mut mock = MockAudioCaptureImpl::new();
        mock.expect_start()
            .returning(|_, _| Err(AppError::AudioCapture("no device".into())));

        let config = AudioConfig::default();
        let result = mock.start(&config, dummy_sender());
        assert!(result.is_err());
    }
}
