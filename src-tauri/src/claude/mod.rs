pub mod batch;
pub mod bridge;
pub mod types;

pub use batch::BatchProcessor;
pub use bridge::ClaudeCodeBridge;
pub use types::*;

use std::path::PathBuf;

use crate::error::AppError;

/// Check if the AI sidecar (Node.js) files are available.
pub fn sidecar_available() -> bool {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sidecars")
        .join("ai-bridge");
    let check = |dir: &PathBuf| -> bool {
        dir.join("index.mjs").exists() && dir.join("node_modules").exists()
    };
    if check(&dev_path) {
        return true;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let prod_path = exe_dir.join("sidecars").join("ai-bridge");
            return check(&prod_path.to_path_buf());
        }
    }
    false
}

/// AI analysis trait for text analysis via Claude Code SDK.
///
/// Implementations handle communication with Claude for
/// keyword extraction, summarization, and investigation.
pub trait AiAnalyzer: Send + Sync {
    /// Analyze a batch of transcript text, returning keywords,
    /// summary, action items, and decisions.
    fn analyze_batch(
        &self,
        text: &str,
        topic: Option<&str>,
        existing_keywords: &[String],
    ) -> Result<AnalysisBatchResult, AppError>;

    /// Investigate a specific query using web search.
    fn investigate(&self, query: &str, context: &str) -> Result<InvestigationResult, AppError>;
}
