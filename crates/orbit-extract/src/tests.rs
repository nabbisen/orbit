//! Tests for orbit-extract, validating RFC-005 §18 acceptance cases:
//! normalization rules, paragraph segmentation with exact lines,
//! markdown structure (heading paths, fences), encoding failures,
//! unsupported types, and Japanese text passthrough.

use crate::normalize::normalize_document;
use crate::types::{LocationQuality, SegmentKind};
use crate::{DocumentExtractor, ExtractorRegistry};
use orbit_core::{ErrorCategory, OrbitError, SourceId};
use orbit_fs::ValidatedPath;
use std::fs;
use std::path::Path;

fn validated(path: &Path) -> ValidatedPath {
    ValidatedPath {
        source_id: SourceId::generate(),
        canonical: fs::canonicalize(path).unwrap(),
    }
}

// RFC-005 §9: norm-v1 rules are exact.
#[test]
fn norm_v1_rules() {
    // BOM strip + CRLF + lone CR + trailing space + control chars.
    let input = "\u{FEFF}line one  \r\nline\u{0007} two\rline three\t.\n";
    let normalized = normalize_document(input);
    assert_eq!(normalized, "line one\nline two\nline three\t.\n");
}

// Japanese text passes through unmodified (RFC-014 §5: no lossy
// transformation before indexing).
#[test]
fn norm_v1_preserves_japanese() {
    let input = "日本語のテキスト。\n改行も保持される。\n";
    assert_eq!(normalize_document(input), input);
}

// RFC-005 §18: plain text basic extraction with exact line ranges.
#[test]
fn plain_text_paragraph_lines_exact() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    fs::write(&file, "para one line 1\npara one line 2\n\npara two\n").unwrap();

    let out = ExtractorRegistry::default().extract(&validated(&file)).unwrap();
    assert_eq!(out.extractor_name, "plain_text");
    assert_eq!(out.normalization_version, "norm-v1");
    assert_eq!(out.segments.len(), 2);
    assert_eq!(out.segments[0].line_start, 1);
    assert_eq!(out.segments[0].line_end, 2);
    assert_eq!(out.segments[1].line_start, 4);
    assert_eq!(out.segments[1].line_end, 4);
    assert!(out.segments.iter().all(|s| s.location_quality == LocationQuality::Exact));
}

// RFC-005 §18: markdown headings produce heading paths on following
// content; code fences become code segments with exact ranges.
#[test]
fn markdown_structure_and_heading_paths() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("guide.md");
    fs::write(
        &file,
        "# Guide\n\n## Install\n\nRun the installer.\n\n```sh\ncargo install orbit\n```\n\n## Use\n\nOpen the app.\n",
    )
    .unwrap();

    let out = ExtractorRegistry::default().extract(&validated(&file)).unwrap();
    assert_eq!(out.extractor_name, "markdown");

    let headings: Vec<_> = out
        .segments
        .iter()
        .filter(|s| s.kind == SegmentKind::Heading)
        .collect();
    assert_eq!(headings.len(), 3);

    let install_para = out
        .segments
        .iter()
        .find(|s| s.text == "Run the installer.")
        .unwrap();
    assert_eq!(install_para.heading_path.as_deref(), Some("Guide > Install"));
    assert_eq!(install_para.line_start, 5);

    let code = out
        .segments
        .iter()
        .find(|s| s.kind == SegmentKind::CodeBlock)
        .unwrap();
    assert_eq!(code.text, "cargo install orbit");
    assert_eq!(code.line_start, 7);
    assert_eq!(code.line_end, 9);

    // Sibling heading replaces, not nests: "Use" path is Guide > Use.
    let use_para = out.segments.iter().find(|s| s.text == "Open the app.").unwrap();
    assert_eq!(use_para.heading_path.as_deref(), Some("Guide > Use"));
}

// RFC-005 §18: empty file extracts to zero segments, not an error.
#[test]
fn empty_file_yields_no_segments() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("empty.txt");
    fs::write(&file, "").unwrap();
    let out = ExtractorRegistry::default().extract(&validated(&file)).unwrap();
    assert!(out.segments.is_empty());
    assert_eq!(out.char_count, 0);
}

// RFC-005 §13: invalid UTF-8 is a typed EncodingError.
#[test]
fn invalid_utf8_is_encoding_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.txt");
    fs::write(&file, [0xFFu8, 0xFE, 0x00, 0x41]).unwrap();
    let err = ExtractorRegistry::default().extract(&validated(&file)).unwrap_err();
    match err {
        OrbitError::Extraction { category, .. } => {
            assert_eq!(category, ErrorCategory::EncodingError)
        }
        other => panic!("unexpected error {other:?}"),
    }
}

// RFC-005 §13: unknown extension is a typed UnsupportedType.
#[test]
fn unknown_extension_is_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("image.xyz");
    fs::write(&file, "binaryish").unwrap();
    let err = ExtractorRegistry::default().extract(&validated(&file)).unwrap_err();
    match err {
        OrbitError::Extraction { category, .. } => {
            assert_eq!(category, ErrorCategory::UnsupportedType)
        }
        other => panic!("unexpected error {other:?}"),
    }
}

// Registry selection: markdown wins for .md, plain text takes code.
#[test]
fn registry_selection_by_extension() {
    let registry = ExtractorRegistry::default();
    assert_eq!(registry.select("md").unwrap().name(), "markdown");
    assert_eq!(registry.select("rs").unwrap().name(), "plain_text");
    assert!(registry.select("xyz").is_none());
}

// Unclosed fence terminates at EOF without panicking (malformed input
// robustness, RFC-005 §13).
#[test]
fn unclosed_fence_is_robust() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("broken.md");
    fs::write(&file, "# T\n\n```\nnever closed\n").unwrap();
    let out = ExtractorRegistry::default().extract(&validated(&file)).unwrap();
    let code = out
        .segments
        .iter()
        .find(|s| s.kind == SegmentKind::CodeBlock)
        .unwrap();
    assert_eq!(code.text, "never closed");
}
