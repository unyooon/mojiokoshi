use mockall::mock;

use crate::audio::{AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;

mock! {
    pub AudioCaptureImpl {}

    impl AudioCapture for AudioCaptureImpl {
        fn start(&mut self, config: &AudioConfig) -> Result<(), AppError>;
        fn stop(&mut self) -> Result<(), AppError>;
        fn pause(&mut self) -> Result<(), AppError>;
        fn resume(&mut self) -> Result<(), AppError>;
        fn state(&self) -> CaptureState;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_audio_capture_start_stop() {
        let mut mock = MockAudioCaptureImpl::new();
        mock.expect_start().returning(|_| Ok(()));
        mock.expect_state().returning(|| CaptureState::Capturing);
        mock.expect_stop().returning(|| Ok(()));

        let config = AudioConfig::default();
        assert!(mock.start(&config).is_ok());
        assert_eq!(mock.state(), CaptureState::Capturing);
        assert!(mock.stop().is_ok());
    }

    #[test]
    fn mock_audio_capture_error() {
        let mut mock = MockAudioCaptureImpl::new();
        mock.expect_start()
            .returning(|_| Err(AppError::AudioCapture("no device".into())));

        let config = AudioConfig::default();
        let result = mock.start(&config);
        assert!(result.is_err());
    }
}
