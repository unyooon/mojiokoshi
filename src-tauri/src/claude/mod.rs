/// AI analysis trait for text analysis via Claude Code SDK.
///
/// Implementations handle communication with Claude for
/// keyword extraction, summarization, and investigation.
pub trait AiAnalyzer: Send + Sync {
    /// Analyze transcript text and return structured results.
    fn analyze(&self, text: &str) -> Result<String, crate::error::AppError>;
}
