use std::sync::Mutex;

use tauri::State;

use crate::audio::screen_capture::ScreenCaptureKitCapture;
use crate::audio::{AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::SessionStorage;

pub struct AppState {
    pub capture: Mutex<ScreenCaptureKitCapture>,
    pub storage: Mutex<SqliteStorage>,
}

#[tauri::command]
#[specta::specta]
pub fn health_check() -> Result<String, AppError> {
    Ok(format!(
        "Backend v{} - ok",
        env!("CARGO_PKG_VERSION")
    ))
}

#[tauri::command]
#[specta::specta]
pub fn start_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state
        .capture
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let config = AudioConfig::default();
    capture.start(&config)
}

#[tauri::command]
#[specta::specta]
pub fn stop_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state
        .capture
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    capture.stop()
}

#[tauri::command]
#[specta::specta]
pub fn pause_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state
        .capture
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    capture.pause()
}

#[tauri::command]
#[specta::specta]
pub fn resume_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state
        .capture
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    capture.resume()
}

#[tauri::command]
#[specta::specta]
pub fn get_capture_state(state: State<'_, AppState>) -> Result<CaptureState, AppError> {
    let capture = state
        .capture
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(capture.state())
}

#[tauri::command]
#[specta::specta]
pub fn create_session(
    state: State<'_, AppState>,
    title: String,
) -> Result<String, AppError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    storage.create_session(&title)
}

#[tauri::command]
#[specta::specta]
pub fn end_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    storage.end_session(&session_id)
}

#[tauri::command]
#[specta::specta]
pub fn check_screen_capture_permission() -> Result<bool, AppError> {
    // ScreenCaptureKit permission check will be implemented
    // when screencapturekit-rs is added as a dependency.
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_returns_ok() {
        let result = health_check();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Backend v"));
        assert!(message.contains("- ok"));
    }

    #[test]
    fn test_check_screen_capture_permission_returns_true() {
        let result = check_screen_capture_permission();
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
