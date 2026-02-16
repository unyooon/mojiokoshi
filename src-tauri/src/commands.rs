use std::sync::{Arc, Mutex};

use tauri::State;

use crate::audio::screen_capture::ScreenCaptureKitCapture;
use crate::audio::{AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::SessionStorage;

pub struct AppState {
    pub capture: Mutex<ScreenCaptureKitCapture>,
    pub storage: Arc<SqliteStorage>,
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

#[tauri::command]
#[specta::specta]
pub fn health_check() -> Result<String, AppError> {
    Ok(format!("Backend v{} - ok", env!("CARGO_PKG_VERSION")))
}

#[tauri::command]
#[specta::specta]
pub fn start_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state.capture.lock().map_err(lock_err)?;
    let config = AudioConfig::default();
    capture.start(&config)
}

#[tauri::command]
#[specta::specta]
pub fn stop_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state.capture.lock().map_err(lock_err)?;
    capture.stop()
}

#[tauri::command]
#[specta::specta]
pub fn pause_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state.capture.lock().map_err(lock_err)?;
    capture.pause()
}

#[tauri::command]
#[specta::specta]
pub fn resume_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut capture = state.capture.lock().map_err(lock_err)?;
    capture.resume()
}

#[tauri::command]
#[specta::specta]
pub fn get_capture_state(state: State<'_, AppState>) -> Result<CaptureState, AppError> {
    let capture = state.capture.lock().map_err(lock_err)?;
    Ok(capture.state())
}

#[tauri::command]
#[specta::specta]
pub fn create_session(state: State<'_, AppState>, title: String) -> Result<String, AppError> {
    state.storage.create_session(&title)
}

#[tauri::command]
#[specta::specta]
pub fn end_session(state: State<'_, AppState>, session_id: String) -> Result<(), AppError> {
    state.storage.end_session(&session_id)
}

#[tauri::command]
#[specta::specta]
pub fn check_screen_capture_permission() -> Result<bool, AppError> {
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
