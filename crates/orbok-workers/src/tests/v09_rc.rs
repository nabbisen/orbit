//! v0.9 RC tests: DOCX extractor, HTML extractor, full end-to-end
//! pipeline integration, and pre-release validation gate.

use orbok_cache::CacheService;
use orbok_core::{
    FileStatus, HiddenFilePolicy, IndexMode, JobType, PersistenceMode, SourceType, SymlinkPolicy,
};
use orbok_db::Catalog;
use orbok_db::repo::{
    FileRepository, IndexJobRepository, NewFile, NewSource, ObservedMetadata, SourceRepository,
};
use orbok_extract::{
    ExtractorRegistry,
    types::{DocumentExtractor, LocationQuality, SegmentKind},
};
use orbok_fs::ValidatedPath;
use orbok_search::{HybridSearchService, SearchMode};
use crate::{ChunkAndIndexWorker, ExtractionWorker, run_pending};
use std::fs;
use std::path::PathBuf;

fn catalog_in(root: &std::path::Path) -> (Catalog, CacheService) {
    (Catalog::open(root.join("catalog.sqlite3")).unwrap(), CacheService::new(root))
}

fn validated(path: &std::path::Path) -> ValidatedPath {
    ValidatedPath {
        source_id: orbok_core::SourceId::from_string("s1".to_string()),
        canonical: fs::canonicalize(path).unwrap(),
    }
}

// ── DOCX extractor ────────────────────────────────────────────────────

/// Minimal valid DOCX (ZIP with word/document.xml containing two paragraphs).
fn minimal_docx(content: &str) -> Vec<u8> {
    use std::io::Write;
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
<w:p><w:r><w:t>{}</w:t></w:r></w:p>
<w:p><w:r><w:t>Second paragraph here.</w:t></w:r></w:p>
</w:body></w:document>"#,
        content
    );
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/document.xml", opts).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}

// DOCX extraction: paragraphs extracted, location quality is Approximate.
#[test]
fn docx_extractor_produces_paragraph_segments() {
    use orbok_extract::types::DocumentExtractor;
    let docx_bytes = minimal_docx("Authentication tokens expire after 24 hours.");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.docx");
    fs::write(&path, &docx_bytes).unwrap();
    let vp = validated(&path);
    let out = orbok_extract::registry::ExtractorRegistry::default()
        .extract(&vp)
        .unwrap();
    assert_eq!(out.extractor_name, "docx");
    assert!(!out.segments.is_empty(), "DOCX must produce segments");
    for seg in &out.segments {
        assert_eq!(seg.location_quality, LocationQuality::Approximate,
            "DOCX segments must be Approximate");
        assert_eq!(seg.kind, SegmentKind::Paragraph);
    }
    // Content present
    let combined: String = out.segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(combined.contains("Authentication") || combined.contains("tokens"),
        "extracted text should contain document content: {combined}");
}

// DOCX: missing file returns typed error, no panic.
#[test]
fn docx_extractor_missing_file_returns_error() {
    use orbok_extract::docx::DocxExtractor;
    let vp = ValidatedPath {
        source_id: orbok_core::SourceId::from_string("s1".to_string()),
        canonical: PathBuf::from("/nonexistent/file.docx"),
    };
    assert!(DocxExtractor.extract(&vp).is_err());
}

// DOCX: registered in ExtractorRegistry.
#[test]
fn docx_registered_in_registry() {
    let reg = ExtractorRegistry::default();
    assert_eq!(reg.select("docx").unwrap().name(), "docx");
}

// ── HTML extractor ────────────────────────────────────────────────────

const SAMPLE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title><style>body{color:red}</style></head>
<body>
  <h1>Authentication Guide</h1>
  <p>Tokens expire after <strong>24 hours</strong>. Error code ERR-4042 fires on expiry.</p>
  <h2>Token Rotation</h2>
  <p>The client_secret must be rotated every 90 days.</p>
  <script>alert('ignored')</script>
  <ul><li>Step one</li><li>Step two</li></ul>
</body>
</html>"#;

// HTML: tags stripped, text preserved.
#[test]
fn html_extractor_strips_tags_preserves_text() {
    use orbok_extract::html::HtmlExtractor;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.html");
    fs::write(&path, SAMPLE_HTML).unwrap();
    let out = HtmlExtractor.extract(&validated(&path)).unwrap();
    assert_eq!(out.extractor_name, "html");
    assert!(!out.segments.is_empty());
    let combined: String = out.segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    // Content present, script content absent
    assert!(combined.contains("ERR-4042") || combined.contains("Tokens"),
        "HTML text should be extracted: {combined}");
    assert!(!combined.contains("alert"), "script content must be stripped");
    assert!(!combined.contains("body{color"), "style content must be stripped");
}

// HTML: h1/h2 headings tracked in heading_path.
#[test]
fn html_extractor_tracks_heading_path() {
    use orbok_extract::html::HtmlExtractor;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("guide.html");
    fs::write(&path, SAMPLE_HTML).unwrap();
    let out = HtmlExtractor.extract(&validated(&path)).unwrap();
    let headings: Vec<_> = out.segments.iter()
        .filter(|s| s.kind == SegmentKind::Heading)
        .collect();
    assert!(!headings.is_empty(), "HTML extractor should produce heading segments");
}

// HTML: location quality is Approximate (no byte offsets).
#[test]
fn html_location_quality_is_approximate() {
    use orbok_extract::html::HtmlExtractor;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.html");
    fs::write(&path, SAMPLE_HTML).unwrap();
    let out = HtmlExtractor.extract(&validated(&path)).unwrap();
    for seg in &out.segments {
        assert_ne!(seg.location_quality, LocationQuality::Exact,
            "HTML must not claim Exact location quality");
    }
}

// HTML: registered in registry.
#[test]
fn html_registered_in_registry() {
    let reg = ExtractorRegistry::default();
    assert_eq!(reg.select("html").unwrap().name(), "html");
    assert_eq!(reg.select("htm").unwrap().name(), "html");
}

// ── End-to-end pipeline integration ────────────────────────────────────

// Full pipeline: write files → scan → extract → index → search → results.
// This is the primary RC integration gate.
#[test]
fn e2e_full_pipeline_write_scan_index_search() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = catalog_in(dir.path());
    let root = fs::canonicalize(dir.path()).unwrap();

    // Write diverse file types.
    fs::write(dir.path().join("auth.md"),
        "# Authentication\n\nRefresh tokens expire after 24 hours.\nError code ERR-4042 on missing token.\n").unwrap();
    fs::write(dir.path().join("storage.md"),
        "# Storage\n\nOrbok stores derived indexes not source copies.\ncleanup removes snippet cache.\n").unwrap();
    fs::write(dir.path().join("search.md"),
        "# Search\n\nHybrid search combines FTS5 keyword and vector embeddings via RRF.\n").unwrap();
    fs::write(dir.path().join("config.html"),
        "<h1>Configuration</h1><p>Set client_secret in environment variables.</p>").unwrap();

    // Register source.
    let src = SourceRepository::new(&catalog).insert(NewSource {
        source_type: SourceType::Directory,
        persistence_mode: PersistenceMode::Persistent,
        display_name: Some("e2e-test".into()),
        original_path: root.to_string_lossy().into(),
        canonical_path: root.to_string_lossy().into(),
        index_mode: IndexMode::Balanced,
        include_patterns: vec![],
        exclude_patterns: vec![],
        hidden_file_policy: HiddenFilePolicy::Exclude,
        symlink_policy: SymlinkPolicy::Ignore,
        max_file_size_bytes: None,
    }).unwrap();

    // Scan → enqueue jobs.
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest {
                source_id: src.source_id.clone(),
                force_hash: false,
                enqueue_index_jobs: true,
            },
            &AtomicBool::new(false),
        ).unwrap();
    }

    let pending = IndexJobRepository::new(&catalog).list_queued(100).unwrap();
    assert!(!pending.is_empty(), "scanner must enqueue jobs");

    // Run extraction + indexing pipeline.
    let extract_w = ExtractionWorker::new(&catalog, &cache);
    let chunk_w = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract_w, &chunk_w, None, 200).unwrap();

    // All files indexed — no jobs remaining.
    let remaining = IndexJobRepository::new(&catalog).list_queued(100).unwrap();
    assert!(remaining.is_empty(), "{} jobs still queued after pipeline", remaining.len());

    // Search: specific identifier.
    let search = HybridSearchService::keyword_only(&catalog);
    let results = search.search("ERR-4042", SearchMode::Exact, 10).unwrap();
    assert!(!results.is_empty(), "ERR-4042 must be found");
    assert!(results[0].display_path.contains("auth"),
        "top result for ERR-4042 must be auth.md, got: {}", results[0].display_path);

    // Search: conceptual query finds storage doc.
    let results2 = search.search("snippet cache cleanup", SearchMode::Auto, 10).unwrap();
    assert!(!results2.is_empty(), "cache cleanup query must return results");

    // Search: HTML content indexed.
    let results3 = search.search("client_secret", SearchMode::Exact, 10).unwrap();
    assert!(!results3.is_empty(), "HTML content must be indexed and searchable");
}

// ── Pre-release gate: all file types claimed in docs actually work ──────

#[test]
fn all_documented_file_types_have_extractor() {
    let reg = ExtractorRegistry::default();
    // From docs/src/users/file_types.md supported list.
    let supported = ["txt", "md", "html", "htm", "pdf", "docx", "rs", "py",
                     "js", "ts", "go", "sql", "toml", "yaml", "json"];
    for ext in &supported {
        assert!(reg.select(ext).is_some(),
            "documented extension '.{ext}' has no registered extractor");
    }
}

// Pre-release gate: every extractor has a privacy note in the plugin registry.
#[test]
fn plugin_registry_all_extractors_have_privacy_notes() {
    use orbok_extract::PluginRegistry;
    let reg = PluginRegistry::default();
    assert!(reg.len() >= 5, "expect markdown, docx, html, plain-text, pdf");
    for m in reg.manifests() {
        assert!(!m.privacy_note.is_empty(), "plugin {} missing privacy_note", m.plugin_id);
        assert!(!m.license.is_empty(), "plugin {} missing license", m.plugin_id);
    }
}

// Pre-release gate: startup recovery runs cleanly on empty catalog.
#[test]
fn startup_recovery_clean_on_fresh_catalog() {
    use crate::run_startup_recovery;
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = catalog_in(dir.path());
    let cache_path = dir.path().join("orbok-cache.sqlite3");
    let report = run_startup_recovery(&catalog, &cache_path).unwrap();
    assert_eq!(report.jobs_reset, 0);
    assert_eq!(report.jobs_pending, 0);
    assert!(!report.cache_rebuilt);
}

// Pre-release gate: clean shutdown leaves no running jobs.
#[test]
fn pipeline_leaves_no_running_jobs_after_completion() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = catalog_in(dir.path());
    fs::write(dir.path().join("note.md"), "# Note\nSome content.\n").unwrap();
    let root = fs::canonicalize(dir.path()).unwrap().to_string_lossy().to_string();
    let src = SourceRepository::new(&catalog).insert(NewSource {
        source_type: SourceType::Directory, persistence_mode: PersistenceMode::Persistent,
        display_name: None, original_path: root.clone(), canonical_path: root,
        index_mode: IndexMode::Balanced, include_patterns: vec![], exclude_patterns: vec![],
        hidden_file_policy: HiddenFilePolicy::Exclude, symlink_policy: SymlinkPolicy::Ignore,
        max_file_size_bytes: None,
    }).unwrap();
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest { source_id: src.source_id.clone(),
                           force_hash: false, enqueue_index_jobs: true },
            &AtomicBool::new(false),
        ).unwrap();
    }
    run_pending(&catalog, &ExtractionWorker::new(&catalog, &cache),
                &ChunkAndIndexWorker::new(&catalog, &cache), None, 50).unwrap();
    // No jobs left in running state.
    let conn = catalog.lock();
    let running: i64 = conn.query_row(
        "SELECT COUNT(*) FROM index_jobs WHERE status='running'", [], |r| r.get(0)).unwrap();
    assert_eq!(running, 0, "no jobs should remain in running state after pipeline");
}
