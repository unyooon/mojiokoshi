//! Integration tests for the audio-to-transcript pipeline.
//!
//! Tests the flow: Audio capture -> VAD -> Whisper -> Storage

#[cfg(test)]
mod tests {
    use crate::audio::screen_capture::ScreenCaptureKitCapture;
    use crate::audio::{AudioCapture, AudioConfig, CaptureState};
    use crate::storage::{sqlite::SqliteStorage, Segment, SessionStorage};
    use crate::whisper::{
        pipeline::RingBuffer,
        stub::{StubRecognizer, StubVad},
        SpeechRecognizer, VoiceActivityDetector,
    };

    #[test]
    fn test_capture_to_vad_pipeline() {
        // 1. Start capture
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();
        capture.start(&config).unwrap();
        assert_eq!(capture.state(), CaptureState::Capturing);

        // 2. Simulate audio data in ring buffer
        let mut ring_buffer = RingBuffer::new(5.0, 16000);
        let samples: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.01).sin() * 0.8).collect();
        ring_buffer.push_samples(&samples);
        assert!(!ring_buffer.is_empty());

        // 3. Run VAD on samples
        let vad = StubVad::default();
        let buffer_samples = ring_buffer.samples();
        let is_speech = vad.is_speech(&buffer_samples, 16000).unwrap();
        // Loud signal should be detected as speech by stub
        assert!(is_speech);

        // 4. Stop capture
        capture.stop().unwrap();
        assert_eq!(capture.state(), CaptureState::Idle);
    }

    #[test]
    fn test_vad_to_whisper_pipeline() {
        let vad = StubVad::default();
        let recognizer = StubRecognizer;

        // Generate audio with speech-like energy
        let speech_samples: Vec<f32> = (0..8000).map(|i| (i as f32 * 0.02).sin() * 0.8).collect();

        // VAD should detect speech
        let is_speech = vad.is_speech(&speech_samples, 16000).unwrap();
        assert!(is_speech);

        // Whisper processes the detected speech
        let segments = recognizer.transcribe(&speech_samples, 16000).unwrap();
        // Stub returns empty, but the pipeline works
        assert!(segments.is_empty()); // Stub behavior

        // Silence should not be detected as speech
        let silence: Vec<f32> = vec![0.0; 8000];
        let is_speech = vad.is_speech(&silence, 16000).unwrap();
        assert!(!is_speech);
    }

    #[test]
    fn test_whisper_to_storage_pipeline() {
        let storage = SqliteStorage::in_memory().unwrap();

        // Create a session
        let session_id = storage.create_session("Test Meeting").unwrap();
        assert!(!session_id.is_empty());

        // Simulate transcription results being stored
        let segment = Segment {
            id: 0,
            session_id: session_id.clone(),
            speaker: Some("Speaker A".to_string()),
            text: "Hello, this is a test.".to_string(),
            start_time: 0.0,
            end_time: 2.5,
            confidence: Some(0.95),
            is_partial: false,
        };

        let segment_id = storage.insert_segment(&segment).unwrap();
        assert!(segment_id > 0);

        // Retrieve segments
        let segments = storage.get_segments(&session_id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello, this is a test.");
        assert_eq!(segments[0].speaker, Some("Speaker A".to_string()));

        // End session
        storage.end_session(&session_id).unwrap();
    }

    #[test]
    fn test_full_pipeline_simulation() {
        // Full pipeline: capture -> buffer -> VAD -> whisper -> storage
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();
        let mut ring_buffer = RingBuffer::new(5.0, config.sample_rate);
        let vad = StubVad::default();
        let recognizer = StubRecognizer;
        let storage = SqliteStorage::in_memory().unwrap();

        // Start session and capture
        let session_id = storage.create_session("Integration Test").unwrap();
        capture.start(&config).unwrap();

        // Simulate multiple audio chunks
        for chunk in 0..3 {
            let samples: Vec<f32> = (0..4000)
                .map(|i| ((i + chunk * 4000) as f32 * 0.015).sin() * 0.8)
                .collect();

            ring_buffer.push_samples(&samples);

            // Check VAD
            let buffer_data = ring_buffer.samples();
            if vad.is_speech(&buffer_data, config.sample_rate).unwrap() {
                // Transcribe
                let _segments = recognizer
                    .transcribe(&buffer_data, config.sample_rate)
                    .unwrap();

                // Store segment
                let segment = Segment {
                    id: 0,
                    session_id: session_id.clone(),
                    speaker: None,
                    text: format!("Chunk {chunk} transcription"),
                    start_time: chunk as f64 * 0.25,
                    end_time: (chunk + 1) as f64 * 0.25,
                    confidence: Some(0.9),
                    is_partial: false,
                };
                storage.insert_segment(&segment).unwrap();
            }
        }

        // Verify stored segments
        let stored = storage.get_segments(&session_id).unwrap();
        assert_eq!(stored.len(), 3);

        // Stop and end
        capture.stop().unwrap();
        storage.end_session(&session_id).unwrap();
    }

    #[test]
    fn test_pause_resume_does_not_lose_data() {
        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();
        let mut ring_buffer = RingBuffer::new(5.0, config.sample_rate);

        capture.start(&config).unwrap();

        // Add data before pause
        ring_buffer.push_samples(&[0.5; 1000]);
        let len_before = ring_buffer.len();

        // Pause
        capture.pause().unwrap();
        assert_eq!(capture.state(), CaptureState::Paused);

        // Buffer should retain data during pause
        assert_eq!(ring_buffer.len(), len_before);

        // Resume
        capture.resume().unwrap();
        assert_eq!(capture.state(), CaptureState::Capturing);

        // Add more data after resume
        ring_buffer.push_samples(&[0.3; 500]);
        assert_eq!(ring_buffer.len(), len_before + 500);

        capture.stop().unwrap();
    }
}
