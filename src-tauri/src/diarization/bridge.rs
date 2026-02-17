use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use super::helpers::{find_python, find_sidecar_dir, ie, lock_err, stub_segments};
use super::types::{
    DiarizeErrorPayload, DiarizePayload, DiarizeRequest, DiarizeResponse, DiarizedSegment,
};
use super::SpeakerDiarizer;
use crate::error::AppError;

/// Bridge to the Python pyannote.audio diarization sidecar.
pub struct PyannoteBridge {
    pub(crate) process: Mutex<Option<BridgeProcess>>,
    pub(crate) is_stub: bool,
}

pub(crate) struct BridgeProcess {
    child: Child,
    writer: BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
}

impl Default for PyannoteBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PyannoteBridge {
    pub fn new() -> Self {
        let is_stub = std::env::var("MOJIOKOSHI_DIARIZE_STUB").unwrap_or_default() == "1";
        Self {
            process: Mutex::new(None),
            is_stub,
        }
    }

    /// Start the Python diarization sidecar process.
    pub fn start(&self) -> Result<(), AppError> {
        if self.is_stub {
            return Ok(());
        }
        let mut guard = self.process.lock().map_err(lock_err)?;
        if guard.is_some() {
            return Ok(());
        }
        let sidecar_dir = find_sidecar_dir()?;
        let python = find_python(&sidecar_dir);
        let mut child = Command::new(python)
            .arg("diarize.py")
            .current_dir(&sidecar_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ie(format!("Failed to start diarization bridge: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| ie("No stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| ie("No stdout".into()))?;
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
}

impl SpeakerDiarizer for PyannoteBridge {
    /// Run speaker diarization on an audio file.
    fn diarize(
        &self,
        audio_path: &str,
        num_speakers: Option<u32>,
    ) -> Result<Vec<DiarizedSegment>, AppError> {
        if self.is_stub {
            return Ok(stub_segments());
        }
        let mut guard = self.process.lock().map_err(lock_err)?;
        let bp = guard
            .as_mut()
            .ok_or_else(|| ie("Bridge not started".into()))?;

        let request = DiarizeRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: "diarize".to_string(),
            payload: DiarizePayload {
                audio_path: audio_path.to_string(),
                num_speakers,
            },
        };
        let json = serde_json::to_string(&request).map_err(|e| ie(e.to_string()))?;
        bp.writer
            .write_all(json.as_bytes())
            .map_err(|e| ie(e.to_string()))?;
        bp.writer.write_all(b"\n").map_err(|e| ie(e.to_string()))?;
        bp.writer.flush().map_err(|e| ie(e.to_string()))?;

        let mut line = String::new();
        bp.reader
            .read_line(&mut line)
            .map_err(|e| ie(e.to_string()))?;
        if line.is_empty() {
            return Err(ie("Bridge closed unexpectedly".into()));
        }
        parse_response(&line)
    }
}

impl Drop for PyannoteBridge {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn parse_response(line: &str) -> Result<Vec<DiarizedSegment>, AppError> {
    if let Ok(resp) = serde_json::from_str::<DiarizeResponse>(line) {
        if resp.response_type == "response" {
            return Ok(resp.payload.segments);
        }
    }
    let raw: serde_json::Value =
        serde_json::from_str(line).map_err(|e| ie(format!("Invalid response: {e}")))?;
    if raw.get("type").and_then(|v| v.as_str()) == Some("error") {
        let err: DiarizeErrorPayload =
            serde_json::from_value(raw.get("payload").cloned().unwrap_or_default())
                .map_err(|e| ie(e.to_string()))?;
        return Err(ie(format!("Diarization error: {}", err.message)));
    }
    Err(ie(format!("Unexpected response: {line}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_mode_diarize_returns_segments() {
        let bridge = PyannoteBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        let result = bridge.diarize("/tmp/test.wav", None).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].speaker, "SPEAKER_0");
        assert_eq!(result[1].speaker, "SPEAKER_1");
    }

    #[test]
    fn stub_start_and_stop_are_noop() {
        let bridge = PyannoteBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        assert!(bridge.start().is_ok());
        assert!(bridge.stop().is_ok());
    }

    #[test]
    fn new_creates_instance() {
        let bridge = PyannoteBridge::new();
        let guard = bridge.process.lock().unwrap();
        assert!(guard.is_none());
    }

    #[test]
    fn bridge_lifecycle_stub() {
        let bridge = PyannoteBridge {
            process: Mutex::new(None),
            is_stub: true,
        };
        bridge.start().unwrap();
        let segs = bridge.diarize("/tmp/test.wav", Some(2)).unwrap();
        assert!(!segs.is_empty());
        bridge.stop().unwrap();
    }
}
