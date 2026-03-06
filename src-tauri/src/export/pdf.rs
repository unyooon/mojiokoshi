use std::io::BufWriter;

use printpdf::{Mm, PdfDocument};

use crate::error::AppError;

/// Embedded Roboto Medium font bytes (ASCII-compatible, bundled at compile time).
static ROBOTO_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/RobotoMedium.ttf");

/// A4 page width in millimetres.
const PAGE_WIDTH_MM: f32 = 210.0;
/// A4 page height in millimetres.
const PAGE_HEIGHT_MM: f32 = 297.0;
/// Left/right margin in millimetres.
const MARGIN_MM: f32 = 20.0;
/// Font size for the document title.
const TITLE_FONT_SIZE: f32 = 18.0;
/// Font size for section headings.
const HEADING_FONT_SIZE: f32 = 14.0;
/// Font size for body text.
const BODY_FONT_SIZE: f32 = 11.0;
/// Line height multiplier relative to font size.
const LINE_HEIGHT_FACTOR: f32 = 1.4;

/// Represents a single styled line to be laid out on a PDF page.
#[derive(Debug)]
enum PdfLine {
    /// Document-level title.
    Title(String),
    /// Section heading (e.g., `## Heading`).
    Heading(String),
    /// Plain body text paragraph.
    Body(String),
    /// Blank line used for vertical spacing.
    Blank,
}

/// Parse markdown-like content into a flat list of `PdfLine` items.
///
/// Supports:
/// - `# Title` (heading level 1 → `Title`)
/// - `## Heading` (heading level 2+ → `Heading`)
/// - `**bold**` markers are stripped
/// - Leading `-` / `*` list markers are replaced with `• `
/// - Blank lines become `Blank` entries
///
/// # Parameters
///
/// * `title` - Document title displayed at the top of the first page.
/// * `markdown_content` - Markdown-formatted text to parse.
///
/// # Returns
///
/// A `Vec<PdfLine>` in document order.
fn parse_markdown(title: &str, markdown_content: &str) -> Vec<PdfLine> {
    let mut lines: Vec<PdfLine> = Vec::new();

    lines.push(PdfLine::Title(title.to_string()));
    lines.push(PdfLine::Blank);

    for raw in markdown_content.lines() {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            lines.push(PdfLine::Blank);
            continue;
        }

        // Headings
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let text = strip_bold(rest);
            lines.push(PdfLine::Heading(text));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            let text = strip_bold(rest);
            lines.push(PdfLine::Heading(text));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            let text = strip_bold(rest);
            lines.push(PdfLine::Heading(text));
            continue;
        }

        // Unordered list items
        if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            let text = format!("• {}", strip_bold(rest));
            lines.push(PdfLine::Body(text));
            continue;
        }

        // Plain body
        lines.push(PdfLine::Body(strip_bold(trimmed)));
    }

    lines
}

/// Strip Markdown bold markers (`**text**`) from a string, returning plain text.
fn strip_bold(s: &str) -> String {
    s.replace("**", "")
}

/// Generate a PDF document from a title and Markdown-formatted content.
///
/// The output PDF is A4 size with 20 mm margins. Fonts are embedded (Roboto
/// Medium, ASCII-compatible). Japanese and other non-Latin characters are not
/// rendered in this version.
///
/// # Parameters
///
/// * `title` - Document title, shown at the top of the first page.
/// * `markdown_content` - Markdown-formatted body text.
///
/// # Returns
///
/// Raw PDF bytes that can be saved or transferred as-is.
///
/// # Errors
///
/// Returns `AppError::Internal` when font loading or PDF serialisation fails.
pub fn generate_pdf(title: &str, markdown_content: &str) -> Result<Vec<u8>, AppError> {
    let pdf_lines = parse_markdown(title, markdown_content);

    let (doc, page_idx, layer_idx) =
        PdfDocument::new(title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");

    let font = doc
        .add_external_font(std::io::Cursor::new(ROBOTO_MEDIUM))
        .map_err(|e| AppError::Internal(format!("Failed to load font: {e}")))?;

    let mut current_layer = doc.get_page(page_idx).get_layer(layer_idx);
    let mut y_mm = PAGE_HEIGHT_MM - MARGIN_MM;

    let mut page_count = 1usize;

    for line in &pdf_lines {
        let (text, font_size) = match line {
            PdfLine::Title(t) => (t.as_str(), TITLE_FONT_SIZE),
            PdfLine::Heading(h) => (h.as_str(), HEADING_FONT_SIZE),
            PdfLine::Body(b) => (b.as_str(), BODY_FONT_SIZE),
            PdfLine::Blank => {
                y_mm -= BODY_FONT_SIZE * LINE_HEIGHT_FACTOR * 0.5;
                check_new_page(
                    &doc,
                    &mut current_layer,
                    &mut y_mm,
                    &mut page_count,
                )?;
                continue;
            }
        };

        let line_height = font_size * LINE_HEIGHT_FACTOR;
        check_new_page(&doc, &mut current_layer, &mut y_mm, &mut page_count)?;
        // Filter to ASCII-only to avoid encoding issues (Japanese support planned)
        let safe_text: String = text.chars().filter(|c| c.is_ascii()).collect();

        current_layer.use_text(&safe_text, font_size, Mm(MARGIN_MM), Mm(y_mm), &font);

        y_mm -= line_height;
    }

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)
        .map_err(|e| AppError::Internal(format!("Failed to save PDF: {e}")))?;
    buf.into_inner()
        .map_err(|e| AppError::Internal(format!("Failed to flush PDF buffer: {e}")))
}

/// Add a new page when the y cursor drops below the bottom margin.
///
/// Updates `current_layer` and resets `y_mm` to the top margin on page break.
fn check_new_page(
    doc: &printpdf::PdfDocumentReference,
    current_layer: &mut printpdf::PdfLayerReference,
    y_mm: &mut f32,
    page_count: &mut usize,
) -> Result<(), AppError> {
    if *y_mm < MARGIN_MM {
        *page_count += 1;
        let (new_page, new_layer) = doc.add_page(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            format!("Layer {}", *page_count),
        );
        *current_layer = doc.get_page(new_page).get_layer(new_layer);
        *y_mm = PAGE_HEIGHT_MM - MARGIN_MM;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pdf_returns_non_empty_bytes() {
        let bytes = generate_pdf("Test Title", "## Section\n\nHello world").unwrap();
        assert!(!bytes.is_empty());
        // PDF header magic bytes
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn generate_pdf_empty_content() {
        let bytes = generate_pdf("Empty", "").unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn parse_markdown_title_heading_body() {
        let lines = parse_markdown("My Doc", "## Hello\n\nSome text\n- item");
        assert!(matches!(lines[0], PdfLine::Title(_)));
        // Blank inserted after title
        assert!(matches!(lines[1], PdfLine::Blank));
        assert!(matches!(lines[2], PdfLine::Heading(_)));
        assert!(matches!(lines[3], PdfLine::Blank));
        assert!(matches!(lines[4], PdfLine::Body(_)));
        assert!(matches!(lines[5], PdfLine::Body(ref t) if t.starts_with('•')));
    }

    #[test]
    fn strip_bold_removes_markers() {
        assert_eq!(strip_bold("**bold** text"), "bold text");
        assert_eq!(strip_bold("no markers"), "no markers");
    }

    #[test]
    fn generate_pdf_multipage_content() {
        // Generate enough lines to force a page break
        let body: String = (0..200)
            .map(|i| format!("Line number {i}\n"))
            .collect();
        let bytes = generate_pdf("Long Doc", &body).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }
}
