//! Plain-text extractor, `text-v1` (RFC-005 §5: txt, log, csv, source
//! code as line-aware text).
//!
//! Segmentation: consecutive non-blank lines form a paragraph segment;
//! line ranges are exact. Invalid UTF-8 is a typed `EncodingError`
//! (RFC-005 §13) — extraction never guesses silently.

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
    read_error_category,
};
use orbit_core::{ErrorCategory, OrbitError, OrbitResult, versions::NORMALIZATION_VERSION};
use orbit_fs::ValidatedPath;

pub struct PlainTextExtractor;

impl DocumentExtractor for PlainTextExtractor {
    fn name(&self) -> &'static str {
        "plain_text"
    }

    fn version(&self) -> &'static str {
        "text-v1"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &[
            "txt", "log", "csv", "rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "h", "cpp",
            "hpp", "go", "rb", "php", "sh", "bash", "sql", "toml", "yaml", "yml", "json", "xml",
            "css", "html", "htm",
        ]
    }

    fn extract(&self, path: &ValidatedPath) -> OrbitResult<ExtractOutput> {
        let bytes = std::fs::read(&path.canonical).map_err(|e| OrbitError::Extraction {
            category: read_error_category(&e),
            message: e.to_string(),
        })?;
        let raw = String::from_utf8(bytes).map_err(|_| OrbitError::Extraction {
            category: ErrorCategory::EncodingError,
            message: "file is not valid UTF-8".into(),
        })?;
        let normalized = normalize_document(&raw);
        let segments = segment_paragraphs(&normalized);
        let char_count = normalized.chars().count() as u64;
        Ok(ExtractOutput {
            extractor_name: self.name().into(),
            extractor_version: self.version().into(),
            normalization_version: NORMALIZATION_VERSION.into(),
            segments,
            char_count,
        })
    }
}

/// Group consecutive non-blank lines into paragraph segments with
/// 1-based inclusive line ranges.
pub(crate) fn segment_paragraphs(normalized: &str) -> Vec<ExtractedSegment> {
    let mut segments = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut start_line = 0u32;
    for (idx, line) in normalized.lines().enumerate() {
        let line_no = idx as u32 + 1;
        if line.trim().is_empty() {
            flush(&mut segments, &mut current, start_line, line_no.saturating_sub(1));
        } else {
            if current.is_empty() {
                start_line = line_no;
            }
            current.push(line);
        }
    }
    let last_line = normalized.lines().count() as u32;
    flush(&mut segments, &mut current, start_line, last_line);
    segments
}

fn flush(
    segments: &mut Vec<ExtractedSegment>,
    current: &mut Vec<&str>,
    start: u32,
    end: u32,
) {
    if current.is_empty() {
        return;
    }
    segments.push(ExtractedSegment {
        kind: SegmentKind::Paragraph,
        text: current.join("\n"),
        line_start: start,
        line_end: end,
        heading_path: None,
        location_quality: LocationQuality::Exact,
    });
    current.clear();
}
