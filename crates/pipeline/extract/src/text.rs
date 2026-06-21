//! Plain-text extractor, `text-v1` (RFC-005 §5: txt, log, csv, source
//! code as line-aware text; RFC-044 §16.1 resource limits).
//!
//! Segmentation: consecutive non-blank lines form a paragraph segment;
//! line ranges are exact. Invalid UTF-8 is a typed `EncodingError`
//! (RFC-005 §13) — extraction never guesses silently.

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractContext, ExtractOutput, ExtractWarning, ExtractedSegment,
    LocationKind, LocationQuality, SegmentKind, read_error_category,
};
use orbok_core::{ErrorCategory, OrbokError, OrbokResult, versions::NORMALIZATION_VERSION};
use orbok_fs::ValidatedPath;

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

    fn extract_with_context(
        &self,
        path: &ValidatedPath,
        context: &ExtractContext,
    ) -> OrbokResult<ExtractOutput> {
        let limits = &context.limits;
        let mut warnings = Vec::new();

        // RFC-044 §9.5: check file size before reading.
        let meta = std::fs::metadata(&path.canonical).map_err(|e| OrbokError::Extraction {
            category: read_error_category(&e),
            message: e.to_string(),
        })?;
        if meta.len() > limits.max_file_bytes {
            return Err(OrbokError::Extraction {
                category: ErrorCategory::FileTooLarge,
                message: format!(
                    "file is {} bytes, limit is {}",
                    meta.len(),
                    limits.max_file_bytes
                ),
            });
        }

        let bytes = std::fs::read(&path.canonical).map_err(|e| OrbokError::Extraction {
            category: read_error_category(&e),
            message: e.to_string(),
        })?;
        let raw = String::from_utf8(bytes).map_err(|_| OrbokError::Extraction {
            category: ErrorCategory::EncodingError,
            message: "file is not valid UTF-8".into(),
        })?;

        let normalized = normalize_document(&raw);
        let mut segments = segment_paragraphs(&normalized);
        let mut char_count = normalized.chars().count() as u64;

        // RFC-044 §9.5: extracted char limit — truncate and warn.
        if char_count > limits.max_extracted_chars {
            // Trim to the last segment boundary under the limit.
            let mut kept = 0usize;
            let mut kept_chars = 0u64;
            for seg in &segments {
                let seg_chars = seg.text.chars().count() as u64;
                if kept_chars + seg_chars > limits.max_extracted_chars {
                    break;
                }
                kept_chars += seg_chars;
                kept += 1;
            }
            segments.truncate(kept);
            char_count = kept_chars;
            warnings.push(ExtractWarning::SizeLimitReached {
                limit_name: "max_extracted_chars".into(),
            });
        }

        // RFC-044 §9.5: segment count limit.
        if segments.len() > limits.max_segments {
            segments.truncate(limits.max_segments);
            warnings.push(ExtractWarning::SizeLimitReached {
                limit_name: "max_segments".into(),
            });
        }

        Ok(ExtractOutput {
            extractor_name: self.name().into(),
            extractor_version: self.version().into(),
            normalization_version: NORMALIZATION_VERSION.into(),
            segments,
            char_count,
            warnings,
        })
    }

    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        self.extract_with_context(path, &ExtractContext::default())
    }
}

/// Group consecutive non-blank lines into paragraph segments with
/// 1-based inclusive line ranges and `LocationKind::Lines`.
pub(crate) fn segment_paragraphs(normalized: &str) -> Vec<ExtractedSegment> {
    let mut segments = Vec::new();
    let mut para_lines: Vec<&str> = Vec::new();
    let mut para_start = 0u32;
    let mut line_num = 0u32;

    for line in normalized.lines() {
        line_num += 1;
        if line.trim().is_empty() {
            if !para_lines.is_empty() {
                segments.push(ExtractedSegment {
                    kind: SegmentKind::Paragraph,
                    text: para_lines.join("\n"),
                    line_start: para_start,
                    line_end: line_num - 1,
                    location_kind: LocationKind::Lines,
                    heading_path: None,
                    location_quality: LocationQuality::Exact,
                });
                para_lines.clear();
            }
        } else {
            if para_lines.is_empty() {
                para_start = line_num;
            }
            para_lines.push(line);
        }
    }
    if !para_lines.is_empty() {
        segments.push(ExtractedSegment {
            kind: SegmentKind::Paragraph,
            text: para_lines.join("\n"),
            line_start: para_start,
            line_end: line_num,
            location_kind: LocationKind::Lines,
            heading_path: None,
            location_quality: LocationQuality::Exact,
        });
    }
    segments
}
