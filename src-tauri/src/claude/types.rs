use serde::{Deserialize, Serialize};
use specta::Type;

/// Type of extracted keyword.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum KeywordType {
    TechTerm,
    ProperNoun,
    Acronym,
    Jargon,
}

/// A keyword or term extracted from the transcript.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Keyword {
    pub id: String,
    pub term: String,
    pub keyword_type: KeywordType,
    pub definition: Option<String>,
    pub web_search_result: Option<String>,
    pub source_url: Option<String>,
    pub first_seen_at: f64,
    pub occurrences: u32,
}

/// Rolling summary of the meeting content.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiSummary {
    pub text: String,
    pub updated_at: f64,
    pub covering_from_ms: f64,
    pub covering_to_ms: f64,
}

/// Priority level for action items.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// An action item detected in the meeting.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActionItem {
    pub id: String,
    pub text: String,
    pub assignee: Option<String>,
    pub deadline: Option<String>,
    pub priority: Priority,
    pub completed: bool,
    pub detected_at: f64,
}

/// A decision recorded during the meeting.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Decision {
    pub id: String,
    pub text: String,
    pub context: String,
    pub participants: Vec<String>,
    pub detected_at: f64,
}

/// A source reference from web search.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Source {
    pub title: String,
    pub url: String,
}

/// Result of an on-demand investigation query.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InvestigationResult {
    pub id: String,
    pub query: String,
    pub summary: String,
    pub details: String,
    pub sources: Vec<Source>,
    pub created_at: f64,
}

/// JSON-lines request sent to the Node.js bridge process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub request_type: String,
    pub payload: serde_json::Value,
}

/// JSON-lines response received from the Node.js bridge process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub payload: serde_json::Value,
}

/// Payload for the `analyze_batch` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisBatchPayload {
    pub transcript_text: String,
    pub meeting_topic: Option<String>,
    pub existing_keywords: Vec<String>,
}

/// Result returned from the `analyze_batch` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisBatchResult {
    pub keywords: Vec<Keyword>,
    pub summary: AiSummary,
    pub action_items: Vec<ActionItem>,
    pub decisions: Vec<Decision>,
}

/// Payload for the `investigate` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigatePayload {
    pub query: String,
    pub context: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_type_serialization_round_trip() {
        let types = vec![
            KeywordType::TechTerm,
            KeywordType::ProperNoun,
            KeywordType::Acronym,
            KeywordType::Jargon,
        ];
        for kt in types {
            let json = serde_json::to_string(&kt).unwrap();
            let deserialized: KeywordType = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{kt:?}"));
        }
    }

    #[test]
    fn priority_serialization_round_trip() {
        let priorities = vec![Priority::High, Priority::Medium, Priority::Low];
        for p in priorities {
            let json = serde_json::to_string(&p).unwrap();
            let deserialized: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{p:?}"));
        }
    }

    #[test]
    fn bridge_request_uses_type_field() {
        let req = BridgeRequest {
            id: "test-1".to_string(),
            request_type: "analyze_batch".to_string(),
            payload: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"analyze_batch""#));
        let deserialized: BridgeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_type, "analyze_batch");
    }

    #[test]
    fn bridge_response_uses_type_field() {
        let resp = BridgeResponse {
            id: "test-1".to_string(),
            response_type: "response".to_string(),
            payload: serde_json::json!({"text": "hello"}),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""type":"response""#));
        let deserialized: BridgeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.response_type, "response");
    }

    #[test]
    fn analysis_batch_result_round_trip() {
        let result = AnalysisBatchResult {
            keywords: vec![Keyword {
                id: "kw-1".to_string(),
                term: "Rust".to_string(),
                keyword_type: KeywordType::TechTerm,
                definition: Some("A systems programming language".to_string()),
                web_search_result: None,
                source_url: None,
                first_seen_at: 1000.0,
                occurrences: 3,
            }],
            summary: AiSummary {
                text: "Discussion about Rust".to_string(),
                updated_at: 2000.0,
                covering_from_ms: 0.0,
                covering_to_ms: 5000.0,
            },
            action_items: vec![ActionItem {
                id: "ai-1".to_string(),
                text: "Review the Rust code".to_string(),
                assignee: Some("Alice".to_string()),
                deadline: Some("2026-02-20".to_string()),
                priority: Priority::High,
                completed: false,
                detected_at: 3000.0,
            }],
            decisions: vec![Decision {
                id: "d-1".to_string(),
                text: "Use Rust for the backend".to_string(),
                context: "Performance requirements".to_string(),
                participants: vec!["Alice".to_string(), "Bob".to_string()],
                detected_at: 4000.0,
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AnalysisBatchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.keywords.len(), 1);
        assert_eq!(deserialized.keywords[0].term, "Rust");
        assert_eq!(deserialized.action_items.len(), 1);
        assert_eq!(deserialized.decisions.len(), 1);
    }

    #[test]
    fn investigate_payload_round_trip() {
        let payload = InvestigatePayload {
            query: "What is WebRTC?".to_string(),
            context: "Discussing real-time communication".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: InvestigatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, payload.query);
        assert_eq!(deserialized.context, payload.context);
    }

    #[test]
    fn investigation_result_round_trip() {
        let result = InvestigationResult {
            id: "inv-1".to_string(),
            query: "What is WebRTC?".to_string(),
            summary: "WebRTC is a protocol for real-time communication".to_string(),
            details: "Detailed findings here".to_string(),
            sources: vec![Source {
                title: "MDN Web Docs".to_string(),
                url: "https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API".to_string(),
            }],
            created_at: 5000.0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: InvestigationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sources.len(), 1);
        assert_eq!(deserialized.sources[0].title, "MDN Web Docs");
    }
}
