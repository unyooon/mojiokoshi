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
use crate::whisper::{SpeechRecognizer, VoiceActivityDetector, WhisperModelStatus};

pub struct AppState {
    pub capture: Mutex<ScreenCaptureKitCapture>,
    pub storage: Arc<SqliteStorage>,
    pub pipeline_shutdown: Arc<AtomicBool>,
    pub pipeline_handle: Mutex<Option<JoinHandle<()>>>,
    pub vad: Mutex<Option<Arc<dyn VoiceActivityDetector + Send + Sync>>>,
    pub recognizer: Mutex<Option<Arc<dyn SpeechRecognizer + Send + Sync>>>,
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}
fn model_not_downloaded() -> AppError {
    AppError::SpeechRecognition("Model not downloaded. Please download from Settings.".into())
}
fn tauri_err(e: tauri::Error) -> AppError {
    AppError::Internal(e.to_string())
}
fn config_err(e: tauri::Error) -> AppError {
    AppError::Config(e.to_string())
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
    let vad = state.vad.lock().map_err(lock_err)?;
    let recognizer = state.recognizer.lock().map_err(lock_err)?;
    let vad_arc = vad.as_ref().ok_or_else(model_not_downloaded)?.clone();
    let recognizer_arc = recognizer
        .as_ref()
        .ok_or_else(model_not_downloaded)?
        .clone();
    drop(vad);
    drop(recognizer);

    let mut capture = state.capture.lock().map_err(lock_err)?;
    let (sender, receiver) = std::sync::mpsc::channel::<AudioBuffer>();
    capture.start(&AudioConfig::default(), sender)?;

    state.pipeline_shutdown.store(false, Ordering::Release);
    let handle = spawn_pipeline(
        receiver,
        app_handle,
        Arc::clone(&state.pipeline_shutdown),
        vad_arc,
        recognizer_arc,
    );
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
    state.capture.lock().map_err(lock_err)?.pause()
}

#[tauri::command]
#[specta::specta]
pub fn resume_audio_capture(state: State<'_, AppState>) -> Result<(), AppError> {
    state.capture.lock().map_err(lock_err)?.resume()
}

#[tauri::command]
#[specta::specta]
pub fn get_capture_state(state: State<'_, AppState>) -> Result<CaptureState, AppError> {
    Ok(state.capture.lock().map_err(lock_err)?.state())
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
    Ok(SCShareableContent::get().is_ok())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_mini_view(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("mini-view") {
        if window.is_visible().map_err(tauri_err)? {
            window.hide().map_err(tauri_err)?;
        } else {
            window.show().map_err(tauri_err)?;
            window.set_focus().map_err(tauri_err)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SidecarStatus {
    pub ai_available: bool,
    pub diarization_available: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn check_sidecar_status() -> Result<SidecarStatus, AppError> {
    Ok(SidecarStatus {
        ai_available: crate::claude::sidecar_available(),
        diarization_available: crate::diarization::helpers::sidecar_available(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn focus_main_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_focus().map_err(tauri_err)?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_whisper_model_status(
    app: tauri::AppHandle,
    model: String,
) -> Result<WhisperModelStatus, AppError> {
    let data_dir = app.path().app_data_dir().map_err(config_err)?;
    Ok(crate::whisper::model::check_model(&data_dir, &model))
}

#[tauri::command]
#[specta::specta]
pub async fn download_whisper_model(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    model: String,
) -> Result<String, AppError> {
    let data_dir = app.path().app_data_dir().map_err(config_err)?;
    let path = crate::whisper::model::download_model(&data_dir, &model, |_| {})?;
    let path_str = path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Config("Invalid model path".to_string()))?;
    let vad_path = app
        .path()
        .resource_dir()
        .map_err(config_err)?
        .join("resources/silero_vad.onnx");
    let (r, v) = crate::whisper::load_models(&path_str, &vad_path)?;
    *state.recognizer.lock().map_err(lock_err)? = Some(r);
    *state.vad.lock().map_err(lock_err)? = Some(v);
    Ok(path_str)
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
