use std::path::{Path, PathBuf};

use hf_hub::api::sync::{Api, ApiError};
use hf_hub::Repo;

use super::WhisperModelStatus;
use crate::error::AppError;

const HF_REPO: &str = "ggerganov/whisper.cpp";
const SUPPORTED_MODELS: &[&str] = &[
    "tiny",
    "base",
    "small",
    "medium",
    "large-v3",
    "large-v3-turbo",
];

pub fn model_filename(model_name: &str) -> String {
    format!("ggml-{model_name}.bin")
}

pub fn model_path(data_dir: &Path, model_name: &str) -> PathBuf {
    data_dir.join("models").join(model_filename(model_name))
}

pub fn check_model(data_dir: &Path, model_name: &str) -> WhisperModelStatus {
    let path = model_path(data_dir, model_name);
    if path.exists() {
        WhisperModelStatus::Ready {
            path: path.to_string_lossy().to_string(),
        }
    } else {
        WhisperModelStatus::NotDownloaded
    }
}

pub fn validate_model_name(model_name: &str) -> Result<(), AppError> {
    if SUPPORTED_MODELS.contains(&model_name) {
        Ok(())
    } else {
        Err(AppError::Config(format!(
            "Unsupported model: {model_name}. Supported: {}",
            SUPPORTED_MODELS.join(", ")
        )))
    }
}

fn api_err(e: ApiError) -> AppError {
    AppError::SpeechRecognition(e.to_string())
}

pub fn download_model(
    data_dir: &Path,
    model_name: &str,
    on_progress: impl Fn(f32),
) -> Result<PathBuf, AppError> {
    validate_model_name(model_name)?;

    let dest = model_path(data_dir, model_name);
    if dest.exists() {
        on_progress(1.0);
        return Ok(dest);
    }

    let filename = model_filename(model_name);
    let repo = Repo::model(HF_REPO.to_string());
    let api = Api::new().map_err(api_err)?;
    let api_repo = api.repo(repo);

    on_progress(0.0);
    let cached_path = api_repo.download(&filename).map_err(api_err)?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Storage(e.to_string()))?;
    }

    std::fs::copy(&cached_path, &dest).map_err(|e| AppError::Storage(e.to_string()))?;

    on_progress(1.0);
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn model_filename_formats_correctly() {
        assert_eq!(model_filename("base"), "ggml-base.bin");
        assert_eq!(model_filename("large-v3-turbo"), "ggml-large-v3-turbo.bin");
    }

    #[test]
    fn model_path_joins_correctly() {
        let path = model_path(Path::new("/data"), "small");
        assert_eq!(path, PathBuf::from("/data/models/ggml-small.bin"));
    }

    #[test]
    fn check_model_returns_not_downloaded_for_missing() {
        let status = check_model(Path::new("/nonexistent"), "base");
        assert!(matches!(status, WhisperModelStatus::NotDownloaded));
    }

    #[test]
    fn validate_model_name_accepts_supported() {
        assert!(validate_model_name("tiny").is_ok());
        assert!(validate_model_name("base").is_ok());
        assert!(validate_model_name("large-v3").is_ok());
        assert!(validate_model_name("large-v3-turbo").is_ok());
    }

    #[test]
    fn validate_model_name_rejects_unsupported() {
        let result = validate_model_name("nonexistent");
        assert!(result.is_err());
    }
}
