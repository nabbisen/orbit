//! Pipeline integration tests (RFC-006 §20, RFC-007 §20): exercises
//! the full extract → chunk → FTS-index → search chain with real
//! temporary files and an in-memory catalog.

use crate::{ChunkAndIndexWorker, ExtractionWorker, run_pending};
use orbok_cache::CacheService;
use orbok_core::{
    FileStatus, HiddenFilePolicy, IndexMode, JobType, PersistenceMode, SourceType, SymlinkPolicy,
};
use orbok_db::Catalog;
use orbok_db::repo::{
    FileRepository, IndexJobRepository, NewFile, NewSource, ObservedMetadata, SourceRepository,
};
use orbok_search::SearchService;
use std::fs;

fn setup(root: &std::path::Path) -> (Catalog, CacheService) {
    let catalog = Catalog::open(root.join("catalog.sqlite3")).unwrap();
    let cache = CacheService::new(root);
    (catalog, cache)
}

fn seed_md_file(
    catalog: &Catalog,
    root: &std::path::Path,
    name: &str,
    content: &str,
) -> (orbok_core::SourceId, orbok_core::FileId) {
    let file_path = root.join(name);
    fs::write(&file_path, content).unwrap();
    let canonical = fs::canonicalize(&file_path).unwrap();
    let canonical_str = canonical.to_string_lossy().to_string();
    let root_str = fs::canonicalize(root)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let sources = SourceRepository::new(catalog);
    let src = sources
        .insert(NewSource {
            source_type: SourceType::File,
            persistence_mode: PersistenceMode::Persistent,
            display_name: Some(name.into()),
            original_path: canonical_str.clone(),
            canonical_path: root_str.clone(),
            index_mode: IndexMode::Balanced,
            include_patterns: vec![],
            exclude_patterns: vec![],
            hidden_file_policy: HiddenFilePolicy::Exclude,
            symlink_policy: SymlinkPolicy::Ignore,
            max_file_size_bytes: None,
        })
        .unwrap();

    let files = FileRepository::new(catalog);
    let record = files
        .insert(NewFile {
            source_id: src.source_id.clone(),
            original_path: canonical_str.clone(),
            canonical_path: canonical_str.clone(),
            display_path: name.into(),
            extension: Some("md".into()),
            metadata: ObservedMetadata {
                file_size_bytes: content.len() as u64,
                modified_at: Some("2026-01-01T00:00:00Z".into()),
                platform_file_key: None,
                content_hash: Some("abc".into()),
            },
            status: FileStatus::Discovered,
        })
        .unwrap();

    // Queue extract job.
    IndexJobRepository::new(catalog)
        .enqueue(
            JobType::Extract,
            Some(&src.source_id),
            Some(&record.file_id),
        )
        .unwrap();

    (src.source_id, record.file_id)
}

// RFC-007 §20 test 1/2: index and search a simple text chunk.
#[test]
fn extract_chunk_index_and_search_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    let content = "# Authentication\n\nRefresh tokens expire after 24 hours.\n\nError code ERR-4042 occurs when the token is missing.";
    let (_, file_id) = seed_md_file(&catalog, dir.path(), "auth.md", content);

    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    let succeeded = run_pending(&catalog, &extract, &chunk, None, 50).unwrap();
    assert!(
        succeeded >= 2,
        "expected extract + chunk jobs, got {succeeded}"
    );

    let results = SearchService::new(&catalog)
        .search("refresh tokens", 10)
        .unwrap();
    assert!(!results.is_empty(), "should find refresh tokens");
    assert!(results[0].keyword_rank == 1);
}

// RFC-007 §20 test 4: identifiers like ERR-4042.
#[test]
fn identifier_search_matches() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    seed_md_file(
        &catalog,
        dir.path(),
        "doc.md",
        "Error ERR-4042 is a critical system error.\n",
    );
    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract, &chunk, None, 50).unwrap();

    let r = SearchService::new(&catalog).search("ERR-4042", 10).unwrap();
    assert!(!r.is_empty(), "identifier ERR-4042 should match");
}

// RFC-007 §20 test 6: delete removes keyword hit.
#[test]
fn deleted_file_no_longer_matches() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    let (_, file_id) = seed_md_file(&catalog, dir.path(), "temp.md", "temporary secret phrase\n");
    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract, &chunk, None, 50).unwrap();
    assert!(
        !SearchService::new(&catalog)
            .search("secret phrase", 10)
            .unwrap()
            .is_empty()
    );

    // Mark file deleted — stales chunks.
    FileRepository::new(&catalog)
        .set_status(&file_id, FileStatus::Deleted)
        .unwrap();
    // Manually stale chunks (normally done by scanner + cleanup).
    {
        let conn = catalog.lock();
        conn.execute(
            "UPDATE chunks SET chunk_status='stale' WHERE file_id=?1",
            rusqlite::params![file_id.as_str()],
        )
        .unwrap();
    }
    let r = SearchService::new(&catalog)
        .search("secret phrase", 10)
        .unwrap();
    assert!(
        r.is_empty(),
        "deleted file chunks must not appear in results"
    );
}

// RFC-006 §20 test 11: rechunk failure preserves previous active chunks.
#[test]
fn failed_rechunk_preserves_active_index() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    seed_md_file(
        &catalog,
        dir.path(),
        "stable.md",
        "the quick brown fox jumps over\n",
    );
    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract, &chunk, None, 50).unwrap();

    let before = SearchService::new(&catalog)
        .search("quick brown fox", 10)
        .unwrap()
        .len();
    assert!(before > 0);

    // Simulate a failed chunk job (a non-existent file_id).
    let bad_id = orbok_core::FileId::generate();
    let err = chunk.run(&bad_id);
    assert!(err.is_err());

    // Original results unaffected.
    let after = SearchService::new(&catalog)
        .search("quick brown fox", 10)
        .unwrap()
        .len();
    assert_eq!(
        before, after,
        "previous active index must survive a failed rechunk"
    );
}

// Snippets load from the source file.
#[test]
fn search_results_include_snippets() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    seed_md_file(
        &catalog,
        dir.path(),
        "notes.md",
        "# Notes\n\nImportant: token rotation deadline is Friday.\n",
    );
    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract, &chunk, None, 50).unwrap();

    let results = SearchService::new(&catalog)
        .search("token rotation", 10)
        .unwrap();
    assert!(!results.is_empty());
    let snippet = results[0].snippet.as_deref().unwrap_or("");
    assert!(
        !snippet.is_empty(),
        "snippet should be loaded from source file"
    );
}
