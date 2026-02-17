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

    // ── AI Pipeline Integration Tests ──────────────────────────────

    #[test]
    fn test_stub_bridge_batch_pipeline() {
        use crate::claude::batch::BatchProcessor;
        use crate::claude::bridge::ClaudeCodeBridge;
        use std::sync::{Arc, Mutex};

        // Set up stub bridge + storage with segments
        let bridge = Arc::new(ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        });
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let session_id = storage.create_session("AI Pipeline Test").unwrap();

        for (i, (start, end)) in [(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)].iter().enumerate() {
            storage
                .insert_segment(&Segment {
                    id: 0,
                    session_id: session_id.clone(),
                    speaker: Some("Alice".to_string()),
                    text: format!("Discussing Rust and WebRTC in segment {i}"),
                    start_time: *start,
                    end_time: *end,
                    confidence: Some(0.9),
                    is_partial: false,
                })
                .unwrap();
        }

        // Run batch processor through stub bridge
        let mut processor = BatchProcessor::new(
            Arc::clone(&bridge),
            Arc::clone(&storage),
            session_id.clone(),
        );
        let result = processor.process_batch().unwrap();

        assert!(result.is_some());
        let batch = result.unwrap();
        assert_eq!(batch.summary.text, "Stub summary");

        // Second batch returns None (cursor advanced past existing segments)
        let second = processor.process_batch().unwrap();
        assert!(second.is_none());
    }

    #[test]
    fn test_keyword_dedup_across_batches() {
        use crate::claude::types::{Keyword, KeywordType};
        use std::sync::Arc;

        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let session_id = storage.create_session("Keyword Dedup Test").unwrap();

        // Insert a keyword manually
        let kw = Keyword {
            id: "kw-1".to_string(),
            term: "WebRTC".to_string(),
            keyword_type: KeywordType::Acronym,
            definition: Some("Web Real-Time Communication".to_string()),
            web_search_result: None,
            source_url: None,
            first_seen_at: 0.0,
            occurrences: 1,
        };
        storage.insert_keyword(&session_id, &kw).unwrap();

        // Verify it persists
        let terms = storage.get_keyword_terms(&session_id).unwrap();
        assert_eq!(terms, vec!["WebRTC"]);

        // Increment occurrence (simulating dedup in BatchProcessor)
        storage
            .increment_keyword_occurrence(&session_id, "WebRTC")
            .unwrap();
        let keywords = storage.get_keywords(&session_id).unwrap();
        assert_eq!(keywords[0].occurrences, 2);

        // Duplicate insert is ignored (unique index on session_id + term)
        let kw2 = Keyword {
            id: "kw-2".to_string(),
            ..kw.clone()
        };
        storage.insert_keyword(&session_id, &kw2).unwrap();
        let all = storage.get_keywords(&session_id).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_stub_bridge_investigate_pipeline() {
        use crate::claude::bridge::ClaudeCodeBridge;
        use crate::claude::types::{BridgeRequest, InvestigationResult};
        use std::sync::Mutex;

        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };

        let request = BridgeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "investigate".to_string(),
            payload: serde_json::json!({
                "query": "What is WebRTC?",
                "context": "Discussing real-time protocols"
            }),
        };

        let response = bridge.send_request(request).unwrap();
        assert_eq!(response.response_type, "response");

        // Should be parseable as InvestigationResult
        let result: InvestigationResult = serde_json::from_value(response.payload).unwrap();
        assert_eq!(result.summary, "Stub investigation result.");
    }

    #[test]
    fn test_stub_bridge_generate_minutes_pipeline() {
        use crate::claude::bridge::ClaudeCodeBridge;
        use crate::claude::types::BridgeRequest;
        use std::sync::Mutex;

        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };

        // Create storage with transcript segments
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Minutes Test").unwrap();
        storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.clone(),
                speaker: Some("Alice".to_string()),
                text: "Let's deploy the new feature next week.".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                confidence: Some(0.95),
                is_partial: false,
            })
            .unwrap();

        // Build transcript from storage (mirroring generate_minutes command)
        let segments = storage.get_segments(&session_id).unwrap();
        assert_eq!(segments.len(), 1);
        let transcript: String = segments
            .iter()
            .map(|s| {
                let speaker = s.speaker.as_deref().unwrap_or("Unknown");
                format!("[{speaker}] {}", s.text)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let request = BridgeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "generate_minutes".to_string(),
            payload: serde_json::json!({ "transcript_text": transcript }),
        };

        let response = bridge.send_request(request).unwrap();
        assert_eq!(response.response_type, "response");
        let markdown = response
            .payload
            .get("markdown")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(markdown.contains("Meeting Minutes"));
    }

    #[test]
    fn test_full_ai_pipeline_simulation() {
        // Full pipeline: capture -> buffer -> VAD -> whisper -> storage -> AI analysis
        use crate::claude::batch::BatchProcessor;
        use crate::claude::bridge::ClaudeCodeBridge;
        use std::sync::{Arc, Mutex};

        let mut capture = ScreenCaptureKitCapture::new();
        let config = AudioConfig::default();
        let mut ring_buffer = RingBuffer::new(5.0, config.sample_rate);
        let vad = StubVad::default();
        let recognizer = StubRecognizer;
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());

        // Start session and capture
        let session_id = storage.create_session("Full AI Pipeline Test").unwrap();
        capture.start(&config).unwrap();

        // Simulate multiple audio chunks through VAD -> Whisper -> Storage
        for chunk in 0..3 {
            let samples: Vec<f32> = (0..4000)
                .map(|i| ((i + chunk * 4000) as f32 * 0.015).sin() * 0.8)
                .collect();
            ring_buffer.push_samples(&samples);
            let buffer_data = ring_buffer.samples();

            if vad.is_speech(&buffer_data, config.sample_rate).unwrap() {
                let _segments = recognizer
                    .transcribe(&buffer_data, config.sample_rate)
                    .unwrap();
                let segment = Segment {
                    id: 0,
                    session_id: session_id.clone(),
                    speaker: Some("Speaker".to_string()),
                    text: format!("Discussing WebRTC and Rust in chunk {chunk}"),
                    start_time: chunk as f64 * 1000.0,
                    end_time: (chunk + 1) as f64 * 1000.0,
                    confidence: Some(0.9),
                    is_partial: false,
                };
                storage.insert_segment(&segment).unwrap();
            }
        }

        capture.stop().unwrap();

        // Verify segments stored
        let stored = storage.get_segments(&session_id).unwrap();
        assert_eq!(stored.len(), 3);

        // Now run AI analysis via stub bridge
        let bridge = Arc::new(ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        });
        let mut processor = BatchProcessor::new(
            Arc::clone(&bridge),
            Arc::clone(&storage),
            session_id.clone(),
        );
        let result = processor.process_batch().unwrap();
        assert!(result.is_some());
        let batch = result.unwrap();
        assert_eq!(batch.summary.text, "Stub summary");

        // Cursor advanced; second batch returns None
        let second = processor.process_batch().unwrap();
        assert!(second.is_none());

        // End session
        storage.end_session(&session_id).unwrap();
    }

    // ── Phase 3: Diarization & Speaker Pipeline Integration Tests ────

    #[test]
    fn test_diarization_stub_pipeline() {
        use crate::diarization::bridge::PyannoteBridge;
        use crate::diarization::SpeakerDiarizer;
        use std::sync::Mutex;

        // Create PyannoteBridge in stub mode
        let bridge = PyannoteBridge {
            process: Mutex::new(None),
            is_stub: true,
        };

        // Start bridge (no-op in stub mode)
        bridge.start().unwrap();

        // Run diarize and verify segments returned
        let segments = bridge.diarize("/tmp/test.wav", None).unwrap();
        assert!(!segments.is_empty());
        assert_eq!(segments.len(), 6); // stub: 2 speakers alternating every 5s over 30s
        assert_eq!(segments[0].speaker, "SPEAKER_0");
        assert_eq!(segments[1].speaker, "SPEAKER_1");
        assert!((segments[0].start - 0.0).abs() < f64::EPSILON);
        assert!((segments[0].end - 5.0).abs() < f64::EPSILON);
        assert!((segments[5].end - 30.0).abs() < f64::EPSILON);

        // Verify num_speakers parameter is accepted (stub ignores it)
        let segments2 = bridge.diarize("/tmp/test.wav", Some(3)).unwrap();
        assert_eq!(segments2.len(), 6);

        // Stop bridge (no-op in stub mode)
        bridge.stop().unwrap();
    }

    #[test]
    fn test_speaker_assignment_by_time_overlap() {
        // Create SqliteStorage in-memory
        let storage = SqliteStorage::in_memory().unwrap();

        // Create session
        let session_id = storage.create_session("Speaker Assignment Test").unwrap();

        // Insert transcript segments with known time ranges
        let seg_ids: Vec<i64> = [(0.0, 3.0, "Hello everyone"),
                                  (3.0, 7.0, "Let's discuss the agenda"),
                                  (7.0, 12.0, "I agree with the plan"),
                                  (12.0, 15.0, "Any questions?")]
            .iter()
            .map(|(start, end, text)| {
                storage
                    .insert_segment(&Segment {
                        id: 0,
                        session_id: session_id.clone(),
                        speaker: None,
                        text: text.to_string(),
                        start_time: *start,
                        end_time: *end,
                        confidence: Some(0.9),
                        is_partial: false,
                    })
                    .unwrap()
            })
            .collect();

        // Create speakers
        let spk_alice = storage.insert_speaker(&session_id, "Alice", "#ff0000").unwrap();
        let spk_bob = storage.insert_speaker(&session_id, "Bob", "#00ff00").unwrap();

        // Simulate diarized segments with known time ranges
        use crate::diarization::DiarizedSegment;
        let diarized = vec![
            DiarizedSegment { speaker: "SPEAKER_0".to_string(), start: 0.0, end: 6.0 },
            DiarizedSegment { speaker: "SPEAKER_1".to_string(), start: 6.0, end: 14.0 },
            DiarizedSegment { speaker: "SPEAKER_0".to_string(), start: 14.0, end: 16.0 },
        ];

        // Map diarized speaker labels to our speaker IDs
        let speaker_map: std::collections::HashMap<String, String> = [
            ("SPEAKER_0".to_string(), spk_alice.clone()),
            ("SPEAKER_1".to_string(), spk_bob.clone()),
        ].into_iter().collect();

        // For each transcript segment, find the diarized segment with most overlap
        let segments = storage.get_segments(&session_id).unwrap();
        for seg in &segments {
            let mut best_overlap = 0.0_f64;
            let mut best_speaker = String::new();
            for d in &diarized {
                let overlap_start = seg.start_time.max(d.start);
                let overlap_end = seg.end_time.min(d.end);
                let overlap = (overlap_end - overlap_start).max(0.0);
                if overlap > best_overlap {
                    best_overlap = overlap;
                    best_speaker = d.speaker.clone();
                }
            }
            if !best_speaker.is_empty() {
                let speaker_id = speaker_map.get(&best_speaker).unwrap();
                storage.update_segment_speaker(seg.id, speaker_id).unwrap();
            }
        }

        // Verify assignments by reading speaker_id from DB
        let conn = storage.lock_conn().unwrap();
        let assigned: Vec<(i64, String)> = seg_ids.iter().map(|id| {
            let spk: String = conn
                .query_row(
                    "SELECT speaker_id FROM segments WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap();
            (*id, spk)
        }).collect();
        drop(conn);

        // seg 0 (0.0-3.0) overlaps SPEAKER_0 (0-6) -> Alice
        assert_eq!(assigned[0].1, spk_alice);
        // seg 1 (3.0-7.0) overlaps SPEAKER_0 (0-6) for 3s and SPEAKER_1 (6-14) for 1s -> Alice
        assert_eq!(assigned[1].1, spk_alice);
        // seg 2 (7.0-12.0) overlaps SPEAKER_1 (6-14) fully -> Bob
        assert_eq!(assigned[2].1, spk_bob);
        // seg 3 (12.0-15.0) overlaps SPEAKER_1 (6-14) for 2s and SPEAKER_0 (14-16) for 1s -> Bob
        assert_eq!(assigned[3].1, spk_bob);
    }

    #[test]
    fn test_speaker_crud_lifecycle() {
        // Create storage and session
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Speaker CRUD Test").unwrap();

        // Insert speaker and verify with get_speakers
        let spk_id = storage.insert_speaker(&session_id, "Speaker_0", "#3b82f6").unwrap();
        assert!(!spk_id.is_empty());
        let speakers = storage.get_speakers(&session_id).unwrap();
        assert_eq!(speakers.len(), 1);
        assert_eq!(speakers[0].label, "Speaker_0");
        assert_eq!(speakers[0].color, "#3b82f6");
        assert!(!speakers[0].is_self);

        // Update label and verify
        storage.update_speaker_label(&spk_id, "Alice").unwrap();
        let speakers = storage.get_speakers(&session_id).unwrap();
        assert_eq!(speakers[0].label, "Alice");

        // Insert a segment and assign speaker to it
        let seg_id = storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.clone(),
                speaker: None,
                text: "Hello from Alice".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                confidence: Some(0.95),
                is_partial: false,
            })
            .unwrap();

        storage.update_segment_speaker(seg_id, &spk_id).unwrap();

        // Verify the assignment persists
        let conn = storage.lock_conn().unwrap();
        let stored_spk: String = conn
            .query_row(
                "SELECT speaker_id FROM segments WHERE id = ?1",
                rusqlite::params![seg_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_spk, spk_id);
        drop(conn);

        // Add a second speaker and verify both exist
        let spk_id2 = storage.insert_speaker(&session_id, "Bob", "#22c55e").unwrap();
        let speakers = storage.get_speakers(&session_id).unwrap();
        assert_eq!(speakers.len(), 2);

        // Update second speaker label
        storage.update_speaker_label(&spk_id2, "Robert").unwrap();
        let speakers = storage.get_speakers(&session_id).unwrap();
        let labels: Vec<&str> = speakers.iter().map(|s| s.label.as_str()).collect();
        assert!(labels.contains(&"Alice"));
        assert!(labels.contains(&"Robert"));
    }

    #[test]
    fn test_minutes_generation_with_speakers() {
        use crate::claude::bridge::ClaudeCodeBridge;
        use crate::claude::types::BridgeRequest;
        use std::sync::Mutex;

        // Create storage with session and segments that have speaker_ids
        let storage = SqliteStorage::in_memory().unwrap();
        let session_id = storage.create_session("Minutes with Speakers").unwrap();

        // Create speakers
        let alice_id = storage.insert_speaker(&session_id, "Alice", "#ff0000").unwrap();
        let bob_id = storage.insert_speaker(&session_id, "Bob", "#00ff00").unwrap();

        // Insert segments
        let seg1_id = storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.clone(),
                speaker: Some("Unknown".to_string()),
                text: "Let's start the meeting.".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                confidence: Some(0.95),
                is_partial: false,
            })
            .unwrap();

        let seg2_id = storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.clone(),
                speaker: Some("Unknown".to_string()),
                text: "I have the quarterly report ready.".to_string(),
                start_time: 2.0,
                end_time: 5.0,
                confidence: Some(0.92),
                is_partial: false,
            })
            .unwrap();

        let seg3_id = storage
            .insert_segment(&Segment {
                id: 0,
                session_id: session_id.clone(),
                speaker: Some("Unknown".to_string()),
                text: "Great, let's review it together.".to_string(),
                start_time: 5.0,
                end_time: 8.0,
                confidence: Some(0.90),
                is_partial: false,
            })
            .unwrap();

        // Assign speakers to segments
        storage.update_segment_speaker(seg1_id, &alice_id).unwrap();
        storage.update_segment_speaker(seg2_id, &bob_id).unwrap();
        storage.update_segment_speaker(seg3_id, &alice_id).unwrap();

        // Build a transcript string with speaker labels (mirroring minutes generation)
        let segments = storage.get_segments(&session_id).unwrap();
        let speakers = storage.get_speakers(&session_id).unwrap();

        // Build speaker_id -> label lookup
        let speaker_lookup: std::collections::HashMap<String, String> = speakers
            .iter()
            .map(|s| (s.id.clone(), s.label.clone()))
            .collect();

        // Read speaker_id column via direct query for each segment
        let conn = storage.lock_conn().unwrap();
        let transcript: String = segments
            .iter()
            .map(|s| {
                let speaker_id: Option<String> = conn
                    .query_row(
                        "SELECT speaker_id FROM segments WHERE id = ?1",
                        rusqlite::params![s.id],
                        |row| row.get(0),
                    )
                    .ok();
                let label = speaker_id
                    .as_deref()
                    .and_then(|id| speaker_lookup.get(id))
                    .map(|l| l.as_str())
                    .unwrap_or("Unknown");
                format!("[{label}] {}", s.text)
            })
            .collect::<Vec<_>>()
            .join("\n");
        drop(conn);

        // Verify transcript format
        assert!(transcript.contains("[Alice] Let's start the meeting."));
        assert!(transcript.contains("[Bob] I have the quarterly report ready."));
        assert!(transcript.contains("[Alice] Great, let's review it together."));

        // Send to stub bridge for minutes generation
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let request = BridgeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "generate_minutes".to_string(),
            payload: serde_json::json!({ "transcript_text": transcript }),
        };

        let response = bridge.send_request(request).unwrap();
        assert_eq!(response.response_type, "response");
        let markdown = response
            .payload
            .get("markdown")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(markdown.contains("Meeting Minutes"));
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
