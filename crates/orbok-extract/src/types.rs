//! Extraction types (RFC-005 §6–§8).

use orbok_core::{ErrorCategory, OrbokResult};
use orbok_fs::ValidatedPath;
use serde::{Deserialize, Serialize};

/// Segment classification (RFC-005 §8; feeds RFC-006 chunking).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    Heading,
    Paragraph,
    CodeBlock,
    ListItem,
    Table,
    Other,
}

/// How precise the recorded location is (RFC-006 §8 vocabulary, shared
/// here because extraction produces the locations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationQuality {
    Exact,
    Approximate,
    PageOnly,
    Unknown,
}

/// One extracted, normalized segment with source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedSegment {
    pub kind: SegmentKind,
    /// Normalized text (norm-v1).
    pub text: String,
    /// 1-based inclusive line range in the source file.
    pub line_start: u32,
    pub line_end: u32,
    /// Heading trail ("Guide > Install > Linux"), when structure exists.
    pub heading_path: Option<String>,
    pub location_quality: LocationQuality,
}

/// Extraction result for one file (RFC-005 §7). This payload is cached
/// under the `extract-segments:v1` namespace (Appendix A §7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractOutput {
    pub extractor_name: String,
    pub extractor_version: String,
    pub normalization_version: String,
    pub segments: Vec<ExtractedSegment>,
    pub char_count: u64,
}

/// A document extractor (RFC-005 §6). Implementations must:
/// - read only through the [`ValidatedPath`] they are given;
/// - stream or bound memory (NFR-023);
/// - return typed failure categories, never panic on malformed input.
pub trait DocumentExtractor: Send + Sync {
    /// Stable name recorded in `extraction_records.extractor_name`.
    fn name(&self) -> &'static str;
    /// Version recorded for staleness detection (RFC-005 §9).
    fn version(&self) -> &'static str;
    /// Extensions (lowercase, no dot) this extractor handles.
    fn supported_extensions(&self) -> &'static [&'static str];
    /// Extract and normalize.
    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput>;
}

/// Helper for extractors: classify a read failure (RFC-005 §13).
pub fn read_error_category(e: &std::io::Error) -> ErrorCategory {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => ErrorCategory::PermissionDenied,
        std::io::ErrorKind::NotFound => ErrorCategory::SourceMissing,
        _ => ErrorCategory::ReadError,
    }
}
