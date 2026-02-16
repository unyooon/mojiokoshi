/// Audio capture trait for abstracting audio input sources.
///
/// Implementations should handle platform-specific audio capture
/// (e.g., ScreenCaptureKit on macOS).
pub trait AudioCapture: Send + Sync {
    /// Start capturing audio from the configured source.
    fn start(&mut self) -> Result<(), crate::error::AppError>;

    /// Stop capturing audio.
    fn stop(&mut self) -> Result<(), crate::error::AppError>;

    /// Check if currently capturing.
    fn is_capturing(&self) -> bool;
}
