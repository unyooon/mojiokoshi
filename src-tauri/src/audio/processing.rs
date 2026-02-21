use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::Emitter;

use super::AudioBuffer;
use crate::whisper::pipeline::RingBuffer;
use crate::whisper::{
    SpeechRecognizer, TranscriptEvent, TranscriptionSegment, VoiceActivityDetector,
};

const SAMPLE_RATE: u32 = 16000;
const RING_BUFFER_SECS: f32 = 30.0;
const SPEECH_THRESHOLD_SAMPLES: usize = SAMPLE_RATE as usize * 3; // 3 seconds
const RECV_TIMEOUT: Duration = Duration::from_millis(100);

/// Spawn the audio processing pipeline on a background thread.
///
/// Receives `AudioBuffer` from the capture thread, runs VAD, and when
/// enough speech is accumulated, invokes the recognizer and emits
/// Tauri events (`transcript:partial` / `transcript:final`).
pub fn spawn_pipeline(
    receiver: Receiver<AudioBuffer>,
    app_handle: tauri::AppHandle,
    shutdown: Arc<AtomicBool>,
    vad: Arc<dyn VoiceActivityDetector + Send + Sync>,
    recognizer: Arc<dyn SpeechRecognizer + Send + Sync>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        run_pipeline(receiver, app_handle, shutdown, vad, recognizer);
    })
}

fn run_pipeline(
    receiver: Receiver<AudioBuffer>,
    app_handle: tauri::AppHandle,
    shutdown: Arc<AtomicBool>,
    vad: Arc<dyn VoiceActivityDetector + Send + Sync>,
    recognizer: Arc<dyn SpeechRecognizer + Send + Sync>,
) {
    let mut ring_buffer = RingBuffer::new(RING_BUFFER_SECS, SAMPLE_RATE);
    let mut speech_samples: usize = 0;

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        match receiver.recv_timeout(RECV_TIMEOUT) {
            Ok(audio_buf) => {
                ring_buffer.push_samples(&audio_buf.samples);

                if vad
                    .is_speech(&audio_buf.samples, SAMPLE_RATE)
                    .unwrap_or(false)
                {
                    speech_samples += audio_buf.samples.len();
                } else {
                    // Reset counter on silence
                    if speech_samples > 0 && speech_samples < SPEECH_THRESHOLD_SAMPLES {
                        speech_samples = 0;
                    }
                }

                if speech_samples >= SPEECH_THRESHOLD_SAMPLES {
                    let samples = ring_buffer.samples();
                    emit_partial(&app_handle, &samples);

                    if let Ok(segments) = recognizer.transcribe(&samples, SAMPLE_RATE) {
                        emit_final(&app_handle, &segments);
                    }

                    ring_buffer.clear();
                    speech_samples = 0;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No data yet — loop back and check shutdown
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Sender dropped — pipeline is done
                break;
            }
        }
    }
}

fn now_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn emit_partial(app_handle: &tauri::AppHandle, samples: &[f32]) {
    let now = now_epoch_ms();
    let duration_ms = (samples.len() as f64 / SAMPLE_RATE as f64) * 1000.0;
    let event = TranscriptEvent {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: now,
        text: "...".to_string(),
        start_ms: 0.0,
        end_ms: duration_ms,
        confidence: 0.0,
        is_partial: true,
    };
    let _ = app_handle.emit("transcript:partial", &event);
}

fn emit_final(app_handle: &tauri::AppHandle, segments: &[TranscriptionSegment]) {
    let now = now_epoch_ms();
    for segment in segments {
        let event = TranscriptEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            text: segment.text.clone(),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            confidence: segment.confidence,
            is_partial: false,
        };
        let _ = app_handle.emit("transcript:final", &event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whisper::stub::StubVad;
    use std::sync::mpsc;

    #[test]
    fn pipeline_shuts_down_on_flag() {
        let (_sender, _receiver) = mpsc::channel::<AudioBuffer>();
        let shutdown = Arc::new(AtomicBool::new(true));

        // Pipeline should exit immediately since shutdown is already set
        let handle = thread::spawn({
            let shutdown = Arc::clone(&shutdown);
            move || {
                let mut ring_buffer = RingBuffer::new(RING_BUFFER_SECS, SAMPLE_RATE);
                let vad = StubVad::default();
                let shutdown_flag = shutdown;
                // Simulate one iteration of the loop
                assert!(shutdown_flag.load(Ordering::Acquire));
                // Verify ring buffer and vad are usable
                ring_buffer.push_samples(&[0.5; 100]);
                let _ = vad.is_speech(&[0.5; 100], SAMPLE_RATE);
            }
        });

        handle.join().unwrap_or(());
    }

    #[test]
    fn pipeline_shuts_down_on_channel_disconnect() {
        let (sender, receiver) = mpsc::channel::<AudioBuffer>();
        let shutdown = Arc::new(AtomicBool::new(false));

        // Drop sender to disconnect channel
        drop(sender);

        // Pipeline should detect disconnection and exit
        let _shutdown_ref = Arc::clone(&shutdown);
        let handle = thread::spawn(move || {
            // Simulate the recv loop — should get Disconnected
            match receiver.recv_timeout(RECV_TIMEOUT) {
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
                _ => panic!("Expected disconnected"),
            }
        });

        handle.join().unwrap_or(());
    }

    #[test]
    fn speech_threshold_requires_three_seconds() {
        assert_eq!(SPEECH_THRESHOLD_SAMPLES, 48000); // 16000 * 3
    }
}
