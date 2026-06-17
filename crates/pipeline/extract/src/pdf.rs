//! PDF text extraction via lopdf (RFC-022 §6).
//!
//! ## RFC-022 evaluation
//!
//! Two backends were evaluated:
//!
//! | Backend | Language | Japanese | License | Notes |
//! |---|---|---|---|---|
//! | **lopdf** (selected) | Rust | UTF-8 text only | MIT | Fast, pure Rust, page-level |
//! | pdfium | Rust binding | Full Unicode | Apache 2.0 | Requires native library |
//!
//! **Selected: lopdf** for v0.7. Reasons: pure Rust (no FFI), compiles
//! everywhere, adequate for text-heavy PDFs. Limitation: scanned or
//! image-only PDFs produce no text (location_quality = Unknown).
//!
//! pdfium is tracked as a future backend for richer PDF support once
//! the native dependency packaging is solved (RFC-022 deferred).
//!
//! ## Security (RFC-015 §14)
//!
//! PDF parsing is treated as hostile input. All errors are caught and
//! returned as `OrbokError::Extraction` with category
//! `ParserError` or `EncryptedDocument`. Panics from lopdf's parser are
//! caught via `std::panic::catch_unwind` in the extraction driver
//! (RFC-005 §13 isolation requirement).
//!
//! ## Location quality
//!
//! lopdf reports text at page granularity. All segments carry
//! `LocationQuality::PageOnly`. Line-level offsets are not available;
//! UI must not show line numbers for PDF results.
//!
//! ## Japanese
//!
//! UTF-8 encoded PDFs (common in modern Japanese documents) extract
//! correctly. Legacy SJIS/EUC PDFs may produce garbled text; the
//! extractor does not attempt character-encoding conversion in v0.7.

use crate::normalize::normalize_document as normalize_text;
use orbok_core::versions::NORMALIZATION_VERSION;
use crate::types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
};
use orbok_core::{ErrorCategory, OrbokError, OrbokResult};
use orbok_fs::ValidatedPath;

/// PDF extractor using lopdf (RFC-022).
pub struct PdfExtractor;

const EXTRACTOR_NAME: &str = "pdf-lopdf";
const EXTRACTOR_VERSION: &str = "v1";

impl DocumentExtractor for PdfExtractor {
    fn name(&self) -> &'static str {
        EXTRACTOR_NAME
    }

    fn version(&self) -> &'static str {
        EXTRACTOR_VERSION
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["pdf"]
    }

    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        let doc = lopdf::Document::load(&path.canonical).map_err(|e| {
            let category = if e.to_string().contains("password")
                || e.to_string().contains("encrypt")
            {
                ErrorCategory::EncryptedDocument
            } else {
                ErrorCategory::ParserError
            };
            OrbokError::Extraction {
                category,
                message: format!("lopdf: {e}"),
            }
        })?;

        let mut segments = Vec::new();
        let mut total_chars = 0u64;
        let pages: Vec<(u32, u16)> = doc.page_iter().collect();
        let total_pages = pages.len() as u32;

        for (page_idx, (obj_id, _gen_id)) in pages.iter().enumerate() {
            let page_num = (page_idx + 1) as u32;
            let text = extract_page_text(&doc, *obj_id, page_num)?;
            if text.trim().is_empty() {
                continue;
            }
            let normalized = normalize_text(&text);
            if normalized.trim().is_empty() {
                continue;
            }
            total_chars += normalized.len() as u64;
            segments.push(ExtractedSegment {
                kind: SegmentKind::Other,
                text: normalized,
                line_start: page_num,
                line_end: page_num,
                heading_path: Some(format!("Page {page_num}")),
                location_quality: LocationQuality::PageOnly,
            });
        }

        if segments.is_empty() {
            tracing::debug!(
                path = %path.canonical.display(),
                pages = total_pages,
                "PDF produced no text — may be scanned/image-only"
            );
        }

        Ok(ExtractOutput {
            extractor_name: EXTRACTOR_NAME.to_string(),
            extractor_version: EXTRACTOR_VERSION.to_string(),
            normalization_version: NORMALIZATION_VERSION.to_string(),
            segments,
            char_count: total_chars,
        })
    }
}

/// Extract text from one page, returning an empty string on any error.
///
/// lopdf's `extract_text` returns a `Result<String>`. Errors are
/// swallowed per RFC-005 §13 (failure isolation: one page failure must
/// not stop extraction of the whole document).
fn extract_page_text(
    doc: &lopdf::Document,
    obj_id: u32,
    _page_num: u32,
) -> OrbokResult<String> {
    match doc.extract_text(&[obj_id]) {
        Ok(text) => Ok(text),
        Err(_) => Ok(String::new()), // page-level failure isolation
    }
}

/// Detect whether a PDF appears to be scanned/image-only (RFC-025).
///
/// Returns `true` when the PDF has pages but extracted text is empty.
/// In this case the user should be informed that OCR is needed.
/// orbok v0.7 does not include an OCR engine; OCR is tracked in RFC-025.
pub fn is_scanned_pdf(output: &super::types::ExtractOutput, page_count: usize) -> bool {
    page_count > 0 && output.char_count == 0
}

/// Helper: try to get page count from a PDF without failing.
pub fn pdf_page_count(path: &std::path::Path) -> usize {
    lopdf::Document::load(path)
        .map(|d| d.get_pages().len())
        .unwrap_or(0)
}
