//! RFC-044 §20.3–§20.6: panic isolation, error mapping, location
//! semantics, and boundary invariants.

use crate::types::{DocumentExtractor, ExtractContext, ExtractedChunk, LocationKind};
use crate::{ExtractorRegistry, chunk};
use orbok_core::{ErrorCategory, OrbokError, SourceId};
use orbok_fs::ValidatedPath;
use std::fs;
use std::path::Path;

fn validated(path: &Path) -> ValidatedPath {
    ValidatedPath {
        source_id: SourceId::generate(),
        canonical: fs::canonicalize(path).unwrap(),
    }
}

// ── §20.3 Panic isolation tests ──────────────────────────────────────────

// RFC-044 §20.3: a panicking extractor returns ParserPanic, not a crash.
struct PanickingExtractor;

impl DocumentExtractor for PanickingExtractor {
    fn name(&self) -> &'static str {
        "panic-test"
    }
    fn version(&self) -> &'static str {
        "v0"
    }
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["panic_test"]
    }
    fn extract_with_context(
        &self,
        _path: &ValidatedPath,
        _context: &ExtractContext,
    ) -> orbok_core::OrbokResult<crate::types::ExtractOutput> {
        panic!("intentional test panic in extractor");
    }
    fn extract(
        &self,
        path: &ValidatedPath,
    ) -> orbok_core::OrbokResult<crate::types::ExtractOutput> {
        self.extract_with_context(path, &ExtractContext::default())
    }
}

#[test]
fn panicking_extractor_returns_parser_panic_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.panic_test");
    fs::write(&file, "content").unwrap();

    let vp = validated(&file);
    let extractor = PanickingExtractor;
    let ctx = ExtractContext::default();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        extractor.extract_with_context(&vp, &ctx)
    }));

    // The panic propagated — that's expected when calling the trait method
    // directly. The registry's extract_safely is what catches it.
    // This test verifies extract_safely wraps correctly:
    let registry = crate::registry::ExtractorRegistry::new_with(vec![Box::new(PanickingExtractor)]);
    let safe_result = registry.extract_safely(&vp, &ctx);
    assert!(result.is_err(), "raw call panics");
    match safe_result {
        Err(OrbokError::Extraction { category, .. }) => {
            assert_eq!(category, ErrorCategory::ParserPanic);
        }
        other => panic!("expected ParserPanic, got {other:?}"),
    }
}

// ── §20.4 Error mapping tests ────────────────────────────────────────────

// RFC-044 §20.4: missing file → SourceMissing.
#[test]
fn missing_file_returns_source_missing() {
    // Construct a ValidatedPath that bypasses canonicalization but points nowhere.
    let vp = ValidatedPath {
        source_id: SourceId::generate(),
        canonical: std::path::PathBuf::from("/nonexistent/path/to/file.txt"),
    };
    let result = ExtractorRegistry::default().extract_with_context(&vp, &ExtractContext::default());

    match result {
        Err(OrbokError::Extraction { category, .. }) => {
            assert_eq!(category, ErrorCategory::SourceMissing);
        }
        other => panic!("expected SourceMissing, got {other:?}"),
    }
}

// RFC-044 §20.4: invalid UTF-8 → EncodingError.
#[test]
fn invalid_utf8_returns_encoding_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.txt");
    fs::write(&file, b"\xFF\xFE invalid bytes").unwrap();

    let result = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default());

    match result {
        Err(OrbokError::Extraction { category, .. }) => {
            assert_eq!(category, ErrorCategory::EncodingError);
        }
        other => panic!("expected EncodingError, got {other:?}"),
    }
}

// RFC-044 §20.4: unsupported extension → UnsupportedType.
#[test]
fn unsupported_extension_returns_unsupported_type() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("file.xyz_unknown");
    fs::write(&file, "content").unwrap();

    let result = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default());

    match result {
        Err(OrbokError::Extraction { category, .. }) => {
            assert_eq!(category, ErrorCategory::UnsupportedType);
        }
        other => panic!("expected UnsupportedType, got {other:?}"),
    }
}

// ── §20.5 Location tests ─────────────────────────────────────────────────

// RFC-044 §20.5: plain text segments use LocationKind::Lines.
#[test]
fn plain_text_segments_use_lines_location_kind() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.txt");
    fs::write(&file, "Para one.\n\nPara two.\n").unwrap();

    let output = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default())
        .unwrap();

    for seg in &output.segments {
        assert_eq!(
            seg.location_kind,
            LocationKind::Lines,
            "plain text must use Lines, got {:?}",
            seg.location_kind
        );
    }
}

// RFC-044 §20.5: Markdown segments use LocationKind::Lines.
#[test]
fn markdown_segments_use_lines_location_kind() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "# Heading\n\nContent.\n").unwrap();

    let output = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default())
        .unwrap();

    for seg in &output.segments {
        assert_eq!(seg.location_kind, LocationKind::Lines);
    }
}

// RFC-044 §20.5: HTML segments use LocationKind::Blocks.
#[test]
fn html_segments_use_blocks_location_kind() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.html");
    fs::write(&file, "<p>Hello world.</p><p>Second paragraph.</p>").unwrap();

    let output = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default())
        .unwrap();

    for seg in &output.segments {
        assert_eq!(
            seg.location_kind,
            LocationKind::Blocks,
            "HTML must use Blocks, got {:?}",
            seg.location_kind
        );
    }
}

// RFC-044 §20.5: chunker propagates location_kind from segments.
#[test]
fn chunker_propagates_location_kind() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "# Section\n\nContent paragraph.\n").unwrap();

    let output = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default())
        .unwrap();
    let chunks: Vec<ExtractedChunk> = chunk(&output, "test.md");

    // Parent chunk uses Lines (from Markdown).
    assert_eq!(chunks[0].location_kind, LocationKind::Lines);
}

// ── §20.6 Boundary tests ─────────────────────────────────────────────────

// RFC-044 §14.6: chunker produces ExtractedChunk (no orbok-db dependency).
// Verified structurally: chunk() returns Vec<ExtractedChunk>, which has
// no orbok-db types. If this compiles, the boundary rule is satisfied.
#[test]
fn chunker_returns_extracted_chunk_not_chunk_spec() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.txt");
    fs::write(&file, "Hello.\n").unwrap();

    let output = ExtractorRegistry::default()
        .extract_with_context(&validated(&file), &ExtractContext::default())
        .unwrap();
    let chunks: Vec<ExtractedChunk> = chunk(&output, "test.txt");
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].chunk_kind, "document");
}

// RFC-044 §15.3: orbok-extract tests don't reference model/embedding crates.
// Verified statically: this file has no imports from orbok-embed or
// orbok-models. If this compiles cleanly, the rule holds.
