use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;
use crate::storage::sqlite::SqliteStorage;

/// Options controlling which sections appear in the exported document.
#[derive(Debug, Serialize, Deserialize, Type)]
pub struct ExportOptions {
    pub session_id: String,
    pub include_summary: bool,
    pub include_actions: bool,
    pub include_keywords: bool,
    pub include_transcript: bool,
}

/// The generated export content and suggested filename.
#[derive(Debug, Serialize, Deserialize, Type)]
pub struct ExportResult {
    pub content: String,
    pub filename: String,
}

/// Format seconds as `HH:MM:SS`.
fn format_timestamp(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// Format an RFC 3339 datetime string into a human-readable date/time.
fn format_datetime(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| rfc3339.to_string())
}

/// Build a sanitized filename from the session title and date.
fn build_filename(title: &str, started_at: &str) -> String {
    let date_part = started_at.get(..10).unwrap_or("unknown-date");
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let name = if sanitized.is_empty() {
        "meeting".to_string()
    } else {
        sanitized
    };
    format!("{date_part}_{name}.md")
}

/// Generate a markdown document from session data.
///
/// Fetches session metadata, transcript segments, speakers, and keywords
/// from the database, then assembles them into structured markdown.
pub fn generate_markdown(
    storage: &SqliteStorage,
    options: &ExportOptions,
) -> Result<ExportResult, AppError> {
    let session = storage.get_session(&options.session_id)?;
    let segments = storage.get_segments(&options.session_id)?;
    let speakers = storage.get_speakers(&options.session_id)?;
    let keywords = storage.get_keywords(&options.session_id)?;

    let mut md = String::new();

    // Header
    md.push_str(&format!("# Meeting: {}\n\n", session.title));
    let started = format_datetime(&session.started_at);
    let ended = session
        .ended_at
        .as_deref()
        .map(format_datetime)
        .unwrap_or_else(|| "ongoing".to_string());
    md.push_str(&format!("**Date:** {started} - {ended}\n"));

    if !speakers.is_empty() {
        let names: Vec<&str> = speakers.iter().map(|s| s.label.as_str()).collect();
        md.push_str(&format!("**Speakers:** {}\n", names.join(", ")));
    }
    md.push('\n');

    // Summary section (placeholder -- real AI summary not stored in DB yet)
    if options.include_summary {
        md.push_str("## Summary\n\n");
        md.push_str("_No AI summary available for this session._\n\n");
    }

    // Action items (not persisted in DB yet; show placeholder)
    if options.include_actions {
        md.push_str("## Action Items\n\n");
        md.push_str("_No action items recorded._\n\n");
    }

    // Keywords
    if options.include_keywords && !keywords.is_empty() {
        let terms: Vec<&str> = keywords.iter().map(|k| k.term.as_str()).collect();
        md.push_str("## Keywords\n\n");
        md.push_str(&terms.join(", "));
        md.push_str("\n\n");
    }

    // Transcript
    if options.include_transcript && !segments.is_empty() {
        md.push_str("## Transcript\n\n");
        for seg in &segments {
            let ts = format_timestamp(seg.start_time);
            let speaker = seg.speaker.as_deref().unwrap_or("Unknown");
            md.push_str(&format!("**[{ts}] {speaker}:** {}\n\n", seg.text));
        }
    }

    let filename = build_filename(&session.title, &session.started_at);
    Ok(ExportResult {
        content: md,
        filename,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Segment, SessionStorage};

    fn setup_db() -> (SqliteStorage, String) {
        let db = SqliteStorage::in_memory().unwrap();
        let sid = db.create_session("Design Review").unwrap();
        (db, sid)
    }

    #[test]
    fn format_timestamp_zero() {
        assert_eq!(format_timestamp(0.0), "00:00:00");
    }

    #[test]
    fn format_timestamp_large() {
        assert_eq!(format_timestamp(3661.0), "01:01:01");
    }

    #[test]
    fn format_datetime_valid_rfc3339() {
        let result = format_datetime("2026-02-17T14:30:00+09:00");
        assert_eq!(result, "2026-02-17 14:30");
    }

    #[test]
    fn format_datetime_invalid_returns_original() {
        assert_eq!(format_datetime("not-a-date"), "not-a-date");
    }

    #[test]
    fn build_filename_sanitizes_title() {
        let f = build_filename("My Meeting!", "2026-02-17T00:00:00Z");
        assert_eq!(f, "2026-02-17_My_Meeting_.md");
    }

    #[test]
    fn build_filename_empty_title() {
        let f = build_filename("", "2026-01-01T00:00:00Z");
        assert_eq!(f, "2026-01-01_meeting.md");
    }

    #[test]
    fn generate_markdown_empty_session() {
        let (db, sid) = setup_db();
        let opts = ExportOptions {
            session_id: sid,
            include_summary: false,
            include_actions: false,
            include_keywords: false,
            include_transcript: true,
        };
        let result = generate_markdown(&db, &opts).unwrap();
        assert!(result.content.contains("# Meeting: Design Review"));
        assert!(!result.content.contains("## Transcript"));
        assert!(result.filename.ends_with(".md"));
    }

    #[test]
    fn generate_markdown_with_segments() {
        let (db, sid) = setup_db();
        db.insert_segment(&Segment {
            id: 0,
            session_id: sid.clone(),
            speaker: Some("Alice".into()),
            text: "Hello everyone".into(),
            start_time: 0.0,
            end_time: 2.0,
            confidence: None,
            is_partial: false,
        })
        .unwrap();
        let opts = ExportOptions {
            session_id: sid,
            include_summary: false,
            include_actions: false,
            include_keywords: false,
            include_transcript: true,
        };
        let result = generate_markdown(&db, &opts).unwrap();
        assert!(result.content.contains("## Transcript"));
        assert!(result
            .content
            .contains("**[00:00:00] Alice:** Hello everyone"));
    }

    #[test]
    fn generate_markdown_with_speakers_and_keywords() {
        let (db, sid) = setup_db();
        db.insert_speaker(&sid, "Alice", "#ff0000").unwrap();
        db.insert_speaker(&sid, "Bob", "#00ff00").unwrap();
        let kw = crate::claude::types::Keyword {
            id: "kw-1".into(),
            term: "Rust".into(),
            keyword_type: crate::claude::types::KeywordType::TechTerm,
            definition: None,
            web_search_result: None,
            source_url: None,
            first_seen_at: 0.0,
            occurrences: 1,
        };
        db.insert_keyword(&sid, &kw).unwrap();
        let opts = ExportOptions {
            session_id: sid,
            include_summary: false,
            include_actions: false,
            include_keywords: true,
            include_transcript: false,
        };
        let result = generate_markdown(&db, &opts).unwrap();
        assert!(result.content.contains("**Speakers:** Alice, Bob"));
        assert!(result.content.contains("## Keywords"));
        assert!(result.content.contains("Rust"));
    }

    #[test]
    fn generate_markdown_all_sections() {
        let (db, sid) = setup_db();
        let opts = ExportOptions {
            session_id: sid,
            include_summary: true,
            include_actions: true,
            include_keywords: true,
            include_transcript: true,
        };
        let result = generate_markdown(&db, &opts).unwrap();
        assert!(result.content.contains("## Summary"));
        assert!(result.content.contains("## Action Items"));
    }

    #[test]
    fn generate_markdown_nonexistent_session() {
        let db = SqliteStorage::in_memory().unwrap();
        let opts = ExportOptions {
            session_id: "no-such-id".into(),
            include_summary: false,
            include_actions: false,
            include_keywords: false,
            include_transcript: false,
        };
        assert!(generate_markdown(&db, &opts).is_err());
    }
}
