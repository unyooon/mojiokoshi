pub mod bridge;
pub mod helpers;
pub mod types;

pub use bridge::PyannoteBridge;
pub use types::*;

use crate::error::AppError;

/// Speaker diarization trait for identifying who spoke when.
///
/// Implementations process audio files and return time-aligned
/// speaker segments (e.g., pyannote.audio pipeline).
pub trait SpeakerDiarizer: Send + Sync {
    /// Run speaker diarization on the given audio file.
    fn diarize(
        &self,
        audio_path: &str,
        num_speakers: Option<u32>,
    ) -> Result<Vec<DiarizedSegment>, AppError>;
}
