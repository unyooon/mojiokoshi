use std::sync::{Arc, Mutex};

use tauri::State;

use crate::diarization::types::SpeakerInfo;
use crate::diarization::{PyannoteBridge, SpeakerDiarizer};
use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;

pub struct DiarizationState {
    pub bridge: Mutex<PyannoteBridge>,
}

fn lock_err<T: std::fmt::Display>(e: T) -> AppError {
    AppError::Internal(format!("Lock poisoned: {e}"))
}

#[tauri::command]
#[specta::specta]
pub fn start_diarization(state: State<'_, DiarizationState>) -> Result<(), AppError> {
    let bridge = state.bridge.lock().map_err(lock_err)?;
    bridge.start()
}

#[tauri::command]
#[specta::specta]
pub fn stop_diarization(state: State<'_, DiarizationState>) -> Result<(), AppError> {
    let bridge = state.bridge.lock().map_err(lock_err)?;
    bridge.stop()
}

#[tauri::command]
#[specta::specta]
pub fn run_diarization(
    state: State<'_, DiarizationState>,
    audio_path: String,
    num_speakers: Option<u32>,
) -> Result<Vec<crate::diarization::DiarizedSegment>, AppError> {
    let bridge = state.bridge.lock().map_err(lock_err)?;
    bridge.diarize(&audio_path, num_speakers)
}

#[tauri::command]
#[specta::specta]
pub fn get_speakers(
    storage: State<'_, Arc<SqliteStorage>>,
    session_id: String,
) -> Result<Vec<SpeakerInfo>, AppError> {
    storage.get_speakers(&session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_bridge() -> PyannoteBridge {
        PyannoteBridge {
            process: Mutex::new(None),
            is_stub: true,
        }
    }

    #[test]
    fn diarization_state_can_be_created() {
        let state = DiarizationState {
            bridge: Mutex::new(stub_bridge()),
        };
        let guard = state.bridge.lock().unwrap();
        assert!(guard.is_stub);
    }

    #[test]
    fn stub_bridge_start_stop() {
        let bridge = stub_bridge();
        assert!(bridge.start().is_ok());
        assert!(bridge.stop().is_ok());
    }

    #[test]
    fn stub_bridge_diarize() {
        let bridge = stub_bridge();
        let segments = bridge.diarize("/tmp/test.wav", Some(2)).unwrap();
        assert!(!segments.is_empty());
        assert_eq!(segments[0].speaker, "SPEAKER_0");
    }

    #[test]
    fn get_speakers_from_storage() {
        let storage = SqliteStorage::in_memory().unwrap();
        use crate::storage::SessionStorage;
        let sid = storage.create_session("Test").unwrap();
        storage.insert_speaker(&sid, "Alice", "#ff0000").unwrap();
        let speakers = storage.get_speakers(&sid).unwrap();
        assert_eq!(speakers.len(), 1);
        assert_eq!(speakers[0].label, "Alice");
    }
}
