use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use crate::error::AppError;

use super::types::{AiSummary, AnalysisBatchResult, BridgeRequest, BridgeResponse};

/// Bridge to the Node.js Claude Agent SDK sidecar process.
/// Communicates via JSON-lines over stdin/stdout.
pub struct ClaudeCodeBridge {
    pub(crate) process: Mutex<Option<BridgeProcess>>,
    pub(crate) is_stub: bool,
}

pub(crate) struct BridgeProcess {
    child: Child,
    writer: BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
}

impl Default for ClaudeCodeBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCodeBridge {
    /// Create a new bridge. Set `MOJIOKOSHI_AI_STUB=1` for mock data.
    pub fn new() -> Self {
        let is_stub = std::env::var("MOJIOKOSHI_AI_STUB").unwrap_or_default() == "1";
        Self {
            process: Mutex::new(None),
            is_stub,
        }
    }

    /// Start the Node.js sidecar process.
    pub fn start(&self) -> Result<(), AppError> {
        if self.is_stub {
            return Ok(());
        }

        let mut guard = self.process.lock().map_err(lock_err)?;

        if guard.is_some() {
            return Ok(());
        }

        let sidecar_dir = find_sidecar_dir()?;

        let mut child = Command::new("node")
            .arg("index.mjs")
            .current_dir(&sidecar_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::AiAnalysis(format!("Failed to start AI bridge: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::AiAnalysis("Failed to capture bridge stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::AiAnalysis("Failed to capture bridge stdout".to_string()))?;

        *guard = Some(BridgeProcess {
            child,
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
        });

        Ok(())
    }

    /// Stop the sidecar process.
    pub fn stop(&self) -> Result<(), AppError> {
        let mut guard = self.process.lock().map_err(lock_err)?;

        if let Some(mut bp) = guard.take() {
            let _ = bp.child.kill();
            let _ = bp.child.wait();
        }
        Ok(())
    }

    /// Send a request and wait for the response.
    pub fn send_request(&self, request: BridgeRequest) -> Result<BridgeResponse, AppError> {
        if self.is_stub {
            return self.stub_response(&request);
        }

        let mut guard = self.process.lock().map_err(lock_err)?;
        let bp = guard
            .as_mut()
            .ok_or_else(|| AppError::AiAnalysis("AI bridge not started".to_string()))?;

        let json =
            serde_json::to_string(&request).map_err(|e| AppError::AiAnalysis(e.to_string()))?;

        bp.writer
            .write_all(json.as_bytes())
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?;
        bp.writer
            .write_all(b"\n")
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?;
        bp.writer
            .flush()
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

        let mut line = String::new();
        bp.reader
            .read_line(&mut line)
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?;

        if line.is_empty() {
            return Err(AppError::AiAnalysis("Bridge closed unexpectedly".into()));
        }

        serde_json::from_str(&line).map_err(|e| AppError::AiAnalysis(e.to_string()))
    }

    fn stub_response(&self, request: &BridgeRequest) -> Result<BridgeResponse, AppError> {
        let payload = match request.request_type.as_str() {
            "analyze_batch" => serde_json::to_value(AnalysisBatchResult {
                keywords: vec![],
                summary: AiSummary {
                    text: "Stub summary".to_string(),
                    updated_at: 0.0,
                    covering_from_ms: 0.0,
                    covering_to_ms: 0.0,
                },
                action_items: vec![],
                decisions: vec![],
            })
            .map_err(|e| AppError::AiAnalysis(e.to_string()))?,
            "investigate" => serde_json::json!({
                "id": format!("inv-{}", request.id),
                "query": "",
                "summary": "Stub investigation result.",
                "details": "No real investigation in stub mode.",
                "sources": [],
                "created_at": 0.0
            }),
            "generate_minutes" => serde_json::json!({
                "markdown": "# Meeting Minutes (Stub)\n\nNo content."
            }),
            "translate" => {
                let text = request
                    .payload
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                serde_json::json!({
                    "translated_text": format!("[Translation] {text}")
                })
            }
            other => {
                return Err(AppError::AiAnalysis(format!(
                    "Unknown request type: {other}"
                )))
            }
        };

        Ok(BridgeResponse {
            id: request.id.clone(),
            response_type: "response".to_string(),
            payload,
        })
    }
}

impl Drop for ClaudeCodeBridge {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Locate the sidecar directory relative to the executable.
fn find_sidecar_dir() -> Result<PathBuf, AppError> {
    // In development, use the source tree path
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sidecars")
        .join("ai-bridge");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    // In production, look next to the executable
    let exe = std::env::current_exe().map_err(|e| AppError::AiAnalysis(e.to_string()))?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| AppError::AiAnalysis("No parent dir".to_string()))?;
    let prod_path = exe_dir.join("sidecars").join("ai-bridge");
    if prod_path.exists() {
        return Ok(prod_path);
    }

    Err(AppError::AiAnalysis(format!(
        "Sidecar directory not found at {dev_path:?} or {prod_path:?}"
    )))
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_instance() {
        let bridge = ClaudeCodeBridge::new();
        assert!(!bridge.is_stub || bridge.is_stub); // just verify it's created
    }

    #[test]
    fn stub_mode_returns_analyze_batch() {
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let req = BridgeRequest {
            id: "test-1".to_string(),
            request_type: "analyze_batch".to_string(),
            payload: serde_json::json!({}),
        };
        let resp = bridge.send_request(req).unwrap();
        assert_eq!(resp.id, "test-1");
        assert_eq!(resp.response_type, "response");

        let result: AnalysisBatchResult = serde_json::from_value(resp.payload).unwrap();
        assert_eq!(result.summary.text, "Stub summary");
        assert!(result.keywords.is_empty());
    }

    #[test]
    fn stub_mode_returns_investigate() {
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let req = BridgeRequest {
            id: "test-2".to_string(),
            request_type: "investigate".to_string(),
            payload: serde_json::json!({}),
        };
        let resp = bridge.send_request(req).unwrap();
        assert_eq!(resp.response_type, "response");
        let summary = resp.payload.get("summary").and_then(|v| v.as_str());
        assert_eq!(summary, Some("Stub investigation result."));
    }

    #[test]
    fn stub_mode_returns_generate_minutes() {
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let req = BridgeRequest {
            id: "test-3".to_string(),
            request_type: "generate_minutes".to_string(),
            payload: serde_json::json!({}),
        };
        let resp = bridge.send_request(req).unwrap();
        assert_eq!(resp.response_type, "response");
        let markdown = resp.payload.get("markdown").and_then(|v| v.as_str());
        assert!(markdown.is_some());
    }

    #[test]
    fn stub_mode_errors_on_unknown_type() {
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let req = BridgeRequest {
            id: "test-4".to_string(),
            request_type: "nonexistent".to_string(),
            payload: serde_json::json!({}),
        };
        let result = bridge.send_request(req);
        assert!(result.is_err());
    }

    #[test]
    fn stub_start_and_stop_are_noop() {
        let bridge = ClaudeCodeBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        assert!(bridge.start().is_ok());
        assert!(bridge.stop().is_ok());
    }

    #[test]
    fn find_sidecar_dir_succeeds() {
        let result = find_sidecar_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}
