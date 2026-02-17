use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use screencapturekit::prelude::*;

use crate::audio::sample_convert::bytes_to_f32_samples;
use crate::audio::AudioBuffer;

const SAMPLE_RATE: u32 = 16000;

/// Handles audio output from an SCStream capture session.
///
/// Receives `CMSampleBuffer` callbacks, extracts f32 audio samples,
/// and forwards them via an `mpsc::Sender`.
pub struct AudioOutputHandler {
    sender: Sender<AudioBuffer>,
    paused: Arc<AtomicBool>,
}

impl AudioOutputHandler {
    pub fn new(sender: Sender<AudioBuffer>, paused: Arc<AtomicBool>) -> Self {
        Self { sender, paused }
    }
}

impl SCStreamOutputTrait for AudioOutputHandler {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }
        if self.paused.load(Ordering::Acquire) {
            return;
        }

        let Some(audio_buffers) = sample_buffer.audio_buffer_list() else {
            return;
        };

        for buf in &audio_buffers {
            let data = buf.data();
            if data.is_empty() {
                continue;
            }
            let samples = bytes_to_f32_samples(data);
            if samples.is_empty() {
                continue;
            }

            let ts = sample_buffer.presentation_timestamp();
            let timestamp_ms = if ts.timescale > 0 {
                (ts.value as f64 / ts.timescale as f64) * 1000.0
            } else {
                0.0
            };

            let buffer = AudioBuffer {
                samples,
                sample_rate: SAMPLE_RATE,
                timestamp_ms,
            };

            // Ignore send errors — the receiver may have been dropped.
            let _ = self.sender.send(buffer);
        }
    }
}
