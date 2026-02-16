pub mod batch;
pub mod bridge;
pub mod types;

pub use batch::BatchProcessor;
pub use bridge::ClaudeCodeBridge;
pub use types::*;

use crate::error::AppError;

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
