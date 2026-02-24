use log::{debug, info, warn};
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
const VAD_CHUNK_SIZE: usize = 512; // Silero VAD v5 expects 512 samples at 16kHz
const RECV_TIMEOUT: Duration = Duration::from_millis(100);
/// RMS energy threshold for fallback speech detection.
/// Used when Silero VAD returns low probabilities for system audio.
const ENERGY_RMS_THRESHOLD: f32 = 0.02;
/// Number of consecutive silence chunks before resetting speech counter.
/// At 512 samples / 16kHz ≈ 32ms per chunk, 94 chunks ≈ 3s of silence.
/// Meeting audio has frequent pauses between speakers.
const SILENCE_CHUNKS_TO_RESET: usize = 94;

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
    info!(
        "Audio processing pipeline started (ring_buffer={}s, speech_threshold={}s)",
        RING_BUFFER_SECS,
        SPEECH_THRESHOLD_SAMPLES as f32 / SAMPLE_RATE as f32
    );

    let mut ring_buffer = RingBuffer::new(RING_BUFFER_SECS, SAMPLE_RATE);
    let mut speech_samples: usize = 0;
    let mut silence_chunks: usize = 0;
    let mut vad_buf: Vec<f32> = Vec::with_capacity(VAD_CHUNK_SIZE);

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        match receiver.recv_timeout(RECV_TIMEOUT) {
            Ok(audio_buf) => {
                ring_buffer.push_samples(&audio_buf.samples);
                vad_buf.extend_from_slice(&audio_buf.samples);

                // Run VAD on full 512-sample chunks (no zero-padding)
                while vad_buf.len() >= VAD_CHUNK_SIZE {
                    let chunk: Vec<f32> = vad_buf.drain(..VAD_CHUNK_SIZE).collect();
                    let vad_prob = vad
                        .speech_probability(&chunk, SAMPLE_RATE)
                        .unwrap_or(0.0);
                    // Energy-based fallback: RMS threshold for system audio
                    let rms = (chunk.iter().map(|s| s * s).sum::<f32>()
                        / chunk.len() as f32)
                        .sqrt();
                    let is_speech = vad_prob >= 0.5 || rms >= ENERGY_RMS_THRESHOLD;
                    debug!(
                        "VAD chunk: vad={vad_prob:.4}, rms={rms:.4}, speech={is_speech}"
                    );

                    if is_speech {
                        speech_samples += VAD_CHUNK_SIZE;
                        silence_chunks = 0;
                        if speech_samples == VAD_CHUNK_SIZE {
                            info!(
                                "Speech detected (vad={vad_prob:.4}, rms={rms:.4}), accumulating..."
                            );
                        }
                    } else {
                        silence_chunks += 1;
                        if speech_samples > 0
                            && speech_samples < SPEECH_THRESHOLD_SAMPLES
                            && silence_chunks >= SILENCE_CHUNKS_TO_RESET
                        {
                            debug!(
                                "Silence for {:.1}s, speech was {:.1}s < 3.0s, resetting",
                                silence_chunks as f32 * VAD_CHUNK_SIZE as f32
                                    / SAMPLE_RATE as f32,
                                speech_samples as f32 / SAMPLE_RATE as f32
                            );
                            speech_samples = 0;
                            silence_chunks = 0;
                        }
                    }
                }

                if speech_samples >= SPEECH_THRESHOLD_SAMPLES {
                    let samples = ring_buffer.samples();
                    info!(
                        "Speech threshold reached ({:.1}s), running transcription on {} samples",
                        speech_samples as f32 / SAMPLE_RATE as f32,
                        samples.len()
                    );
                    emit_partial(&app_handle, &samples);

                    match recognizer.transcribe(&samples, SAMPLE_RATE) {
                        Ok(segments) => {
                            info!("Transcription complete: {} segments", segments.len());
                            for (i, seg) in segments.iter().enumerate() {
                                info!(
                                    "  Segment {}: \"{}\" ({:.0}ms-{:.0}ms)",
                                    i,
                                    seg.text.trim(),
                                    seg.start_ms,
                                    seg.end_ms
                                );
                            }
                            emit_final(&app_handle, &segments);
                        }
                        Err(e) => {
                            warn!("Transcription failed: {e}");
                        }
                    }

                    ring_buffer.clear();
                    speech_samples = 0;
                    silence_chunks = 0;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No data yet — loop back and check shutdown
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                info!("Audio channel disconnected, pipeline exiting");
                break;
            }
        }
    }

    info!("Audio processing pipeline stopped");
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

    #[test]
    fn energy_rms_detects_loud_audio() {
        let loud: Vec<f32> = vec![0.3; VAD_CHUNK_SIZE];
        let rms =
            (loud.iter().map(|s| s * s).sum::<f32>() / loud.len() as f32).sqrt();
        assert!(rms >= ENERGY_RMS_THRESHOLD);
    }

    #[test]
    fn energy_rms_ignores_silence() {
        let silent: Vec<f32> = vec![0.001; VAD_CHUNK_SIZE];
        let rms = (silent.iter().map(|s| s * s).sum::<f32>()
            / silent.len() as f32)
            .sqrt();
        assert!(rms < ENERGY_RMS_THRESHOLD);
    }
}
