pub mod sample_convert;
pub mod screen_capture;
pub mod types;

pub use types::*;

use crate::error::AppError;

/// Audio capture trait for abstracting audio input sources.
///
/// Implementations should handle platform-specific audio capture
/// (e.g., ScreenCaptureKit on macOS).
pub trait AudioCapture: Send + Sync {
    /// Start capturing audio with the given configuration.
    fn start(
        &mut self,
        config: &AudioConfig,
        sender: std::sync::mpsc::Sender<AudioBuffer>,
    ) -> Result<(), AppError>;

    /// Stop capturing audio.
    fn stop(&mut self) -> Result<(), AppError>;

    /// Pause audio capture.
    fn pause(&mut self) -> Result<(), AppError>;

    /// Resume audio capture after pausing.
    fn resume(&mut self) -> Result<(), AppError>;

    /// Get the current capture state.
    fn state(&self) -> CaptureState;
}
