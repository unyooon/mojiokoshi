use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use super::types::{
    DiarizeErrorPayload, DiarizePayload, DiarizeRequest, DiarizeResponse, DiarizedSegment,
};
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

    /// Run speaker diarization on an audio file.
    pub fn diarize(
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

/// Generate stub segments: 2 speakers alternating every 5 seconds (30s total).
fn stub_segments() -> Vec<DiarizedSegment> {
    let (duration, interval) = (30.0_f64, 5.0_f64);
    let mut segments = Vec::new();
    let mut t = 0.0_f64;
    let mut idx = 0u32;
    while t < duration {
        let end = (t + interval).min(duration);
        segments.push(DiarizedSegment {
            speaker: format!("SPEAKER_{idx}"),
            start: t,
            end,
        });
        idx = 1 - idx;
        t = end;
    }
    segments
}

fn find_sidecar_dir() -> Result<PathBuf, AppError> {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sidecars")
        .join("diarization");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    let exe = std::env::current_exe().map_err(|e| ie(e.to_string()))?;
    let exe_dir = exe.parent().ok_or_else(|| ie("No parent dir".into()))?;
    let prod_path = exe_dir.join("sidecars").join("diarization");
    if prod_path.exists() {
        return Ok(prod_path);
    }
    Err(ie(format!(
        "Sidecar not found at {dev_path:?} or {prod_path:?}"
    )))
}

fn find_python(sidecar_dir: &Path) -> String {
    let venv_python = sidecar_dir.join("venv").join("bin").join("python");
    if venv_python.exists() {
        return venv_python.to_string_lossy().to_string();
    }
    "python3".to_string()
}

fn ie(msg: String) -> AppError {
    AppError::Internal(msg)
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_segments_alternates_speakers() {
        let segs = stub_segments();
        assert_eq!(segs.len(), 6);
        assert_eq!(segs[0].speaker, "SPEAKER_0");
        assert_eq!(segs[1].speaker, "SPEAKER_1");
        assert_eq!(segs[2].speaker, "SPEAKER_0");
        assert!((segs[0].start - 0.0).abs() < f64::EPSILON);
        assert!((segs[0].end - 5.0).abs() < f64::EPSILON);
        assert!((segs[5].end - 30.0).abs() < f64::EPSILON);
    }

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
    fn find_sidecar_dir_succeeds() {
        let result = find_sidecar_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
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
