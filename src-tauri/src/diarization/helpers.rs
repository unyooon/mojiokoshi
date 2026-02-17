use std::path::{Path, PathBuf};

use super::types::DiarizedSegment;
use crate::error::AppError;

/// Locate the diarization sidecar directory (dev or production).
pub(crate) fn find_sidecar_dir() -> Result<PathBuf, AppError> {
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

/// Find the Python interpreter, preferring a venv inside the sidecar dir.
pub(crate) fn find_python(sidecar_dir: &Path) -> String {
    let venv_python = sidecar_dir.join("venv").join("bin").join("python");
    if venv_python.exists() {
        return venv_python.to_string_lossy().to_string();
    }
    "python3".to_string()
}

/// Map a poisoned-lock error into `AppError`.
pub(crate) fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

/// Generate stub segments: 2 speakers alternating every 5 seconds (30s total).
pub(crate) fn stub_segments() -> Vec<DiarizedSegment> {
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

pub(crate) fn ie(msg: String) -> AppError {
    AppError::Internal(msg)
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
    fn find_sidecar_dir_succeeds() {
        let result = find_sidecar_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}
