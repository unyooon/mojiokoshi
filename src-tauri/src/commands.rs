use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use tauri::{Manager, State};

use crate::audio::processing::spawn_pipeline;
use crate::audio::screen_capture::ScreenCaptureKitCapture;
use crate::audio::{AudioBuffer, AudioCapture, AudioConfig, CaptureState};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::SessionStorage;

pub struct AppState {
    pub capture: Mutex<ScreenCaptureKitCapture>,
    pub storage: Arc<SqliteStorage>,
    pub pipeline_shutdown: Arc<AtomicBool>,
    pub pipeline_handle: Mutex<Option<JoinHandle<()>>>,
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
pub fn start_audio_capture(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    let mut capture = state.capture.lock().map_err(lock_err)?;
    let config = AudioConfig::default();
    let (sender, receiver) = std::sync::mpsc::channel::<AudioBuffer>();

    capture.start(&config, sender)?;

    state.pipeline_shutdown.store(false, Ordering::Release);
    let handle = spawn_pipeline(receiver, app_handle, Arc::clone(&state.pipeline_shutdown));

    let mut pipeline = state.pipeline_handle.lock().map_err(lock_err)?;
    *pipeline = Some(handle);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn stop_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    state.pipeline_shutdown.store(true, Ordering::Release);

    let mut capture = state.capture.lock().map_err(lock_err)?;
    capture.stop()?;

    let mut pipeline = state.pipeline_handle.lock().map_err(lock_err)?;
    if let Some(handle) = pipeline.take() {
        let _ = handle.join();
    }

    state.pipeline_shutdown.store(false, Ordering::Release);
    Ok(())
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
    use screencapturekit::shareable_content::SCShareableContent;
    match SCShareableContent::get() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_mini_view(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("mini-view") {
        if window
            .is_visible()
            .map_err(|e| AppError::Internal(e.to_string()))?
        {
            window
                .hide()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            window
                .show()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            window
                .set_focus()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn focus_main_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_focus()
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
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
    #[ignore] // Requires macOS with screen capture permission
    fn test_check_screen_capture_permission_returns_bool() {
        let result = check_screen_capture_permission();
        assert!(result.is_ok());
        // Result is either true or false depending on permission state
        let _has_permission: bool = result.unwrap();
    }
}
