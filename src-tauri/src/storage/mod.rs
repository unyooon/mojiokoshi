/// Session storage trait for persisting meeting data.
///
/// Implementations handle CRUD operations for sessions,
/// segments, keywords, and AI insights (e.g., SQLite via rusqlite).
pub trait SessionStorage: Send + Sync {
    /// Create a new session and return its ID.
    fn create_session(&self, title: &str) -> Result<String, crate::error::AppError>;

    /// End a session by setting its end time.
    fn end_session(&self, session_id: &str) -> Result<(), crate::error::AppError>;
}
