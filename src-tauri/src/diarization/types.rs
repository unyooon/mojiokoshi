use serde::{Deserialize, Serialize};
use specta::Type;

/// A single speaker-attributed time segment from diarization.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiarizedSegment {
    pub speaker: String,
    pub start: f64,
    pub end: f64,
}

/// Metadata about a recognized speaker.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SpeakerInfo {
    pub id: String,
    pub label: String,
    pub color: String,
    pub is_self: bool,
}

/// JSON-lines request sent to the diarization sidecar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizeRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub request_type: String,
    pub payload: DiarizePayload,
}

/// Payload for a diarization request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizePayload {
    pub audio_path: String,
    pub num_speakers: Option<u32>,
}

/// JSON-lines response from the diarization sidecar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizeResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub payload: DiarizeResponsePayload,
}

/// Payload of a successful diarization response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizeResponsePayload {
    pub segments: Vec<DiarizedSegment>,
}

/// Error payload returned by the sidecar on failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizeErrorPayload {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diarized_segment_round_trip() {
        let seg = DiarizedSegment {
            speaker: "SPEAKER_0".to_string(),
            start: 0.0,
            end: 2.5,
        };
        let json = serde_json::to_string(&seg).unwrap();
        let de: DiarizedSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(de.speaker, "SPEAKER_0");
        assert!((de.start - 0.0).abs() < f64::EPSILON);
        assert!((de.end - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn speaker_info_round_trip() {
        let info = SpeakerInfo {
            id: "s1".to_string(),
            label: "Alice".to_string(),
            color: "#ff0000".to_string(),
            is_self: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let de: SpeakerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de.label, "Alice");
        assert!(de.is_self);
    }

    #[test]
    fn diarize_request_uses_type_field() {
        let req = DiarizeRequest {
            id: "req-1".to_string(),
            request_type: "diarize".to_string(),
            payload: DiarizePayload {
                audio_path: "/tmp/audio.wav".to_string(),
                num_speakers: Some(2),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"diarize""#));
        let de: DiarizeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.request_type, "diarize");
        assert_eq!(de.payload.num_speakers, Some(2));
    }

    #[test]
    fn diarize_response_round_trip() {
        let resp = DiarizeResponse {
            id: "resp-1".to_string(),
            response_type: "response".to_string(),
            payload: DiarizeResponsePayload {
                segments: vec![
                    DiarizedSegment {
                        speaker: "SPEAKER_0".to_string(),
                        start: 0.0,
                        end: 5.0,
                    },
                    DiarizedSegment {
                        speaker: "SPEAKER_1".to_string(),
                        start: 5.0,
                        end: 10.0,
                    },
                ],
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let de: DiarizeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(de.response_type, "response");
        assert_eq!(de.payload.segments.len(), 2);
        assert_eq!(de.payload.segments[0].speaker, "SPEAKER_0");
    }

    #[test]
    fn diarize_payload_optional_num_speakers() {
        let payload = DiarizePayload {
            audio_path: "/tmp/test.wav".to_string(),
            num_speakers: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains(r#""num_speakers":null"#));
        let de: DiarizePayload = serde_json::from_str(&json).unwrap();
        assert!(de.num_speakers.is_none());
    }
}
