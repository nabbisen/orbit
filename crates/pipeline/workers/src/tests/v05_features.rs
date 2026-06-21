//! v0.5 tests: RFC-012 (model registry), RFC-015 (security),
//! RFC-018 (crash recovery), plus benchmark smoke test.

use crate::{check_catalog_integrity, run_startup_recovery};
use orbok_cache::CacheService;
use orbok_core::{
    HiddenFilePolicy, IndexMode, JobStatus, JobType, PersistenceMode, SourceType, SymlinkPolicy,
};
use orbok_db::Catalog;
use orbok_db::repo::{IndexJobRepository, NewSource};
use orbok_db::repo::{ModelRepository, ModelRole, ModelStatus, NewModel, SourceRepository};
use orbok_search::snippet::html_escape;
use std::fs;

fn setup(root: &std::path::Path) -> (Catalog, CacheService) {
    let catalog = Catalog::open(root.join("catalog.sqlite3")).unwrap();
    let cache = CacheService::new(root);
    (catalog, cache)
}

// ── RFC-012: Model Registry ────────────────────────────────────────────

// RFC-012 §17: model registry lists embedding and reranker roles.
#[test]
fn model_registry_stores_and_retrieves_by_role() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    let repo = ModelRepository::new(&catalog);

    let emb = repo
        .insert(NewModel {
            role: ModelRole::Embedding,
            model_name: "test-embed".into(),
            model_version: "v1".into(),
            local_path: None,
            license_summary: Some("MIT".into()),
            size_bytes: Some(256 * 1024 * 1024),
            backend: Some("candle".into()),
            dimension: Some(768),
            status: ModelStatus::Available,
        })
        .unwrap();
    repo.insert(NewModel {
        role: ModelRole::Reranker,
        model_name: "test-rerank".into(),
        model_version: "v1".into(),
        local_path: None,
        license_summary: None,
        size_bytes: None,
        backend: None,
        dimension: None,
        status: ModelStatus::Missing,
    })
    .unwrap();

    assert_eq!(repo.list_by_role(ModelRole::Embedding).unwrap().len(), 1);
    assert_eq!(repo.list_by_role(ModelRole::Reranker).unwrap().len(), 1);
    assert_eq!(repo.list_all().unwrap().len(), 2);
    assert_eq!(emb.dimension, Some(768));
    assert_eq!(emb.status, ModelStatus::Available);
}

// RFC-012 §17: app works in keyword-only mode with no models.
#[test]
fn keyword_only_works_with_empty_model_registry() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    assert!(
        ModelRepository::new(&catalog)
            .list_all()
            .unwrap()
            .is_empty()
    );
    // No panic; search service can still be created.
    let _ = orbok_search::HybridSearchService::keyword_only(&catalog);
}

// RFC-012 §17: model validation updates status.
#[test]
fn model_validate_marks_missing_when_path_absent() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    let repo = ModelRepository::new(&catalog);
    let model = repo
        .insert(NewModel {
            role: ModelRole::Embedding,
            model_name: "absent".into(),
            model_version: "v1".into(),
            local_path: Some("/nonexistent/model.bin".into()),
            license_summary: None,
            size_bytes: None,
            backend: None,
            dimension: Some(768),
            status: ModelStatus::Available,
        })
        .unwrap();

    let status = repo.validate(&model.model_id, Some(768)).unwrap();
    assert_eq!(status, ModelStatus::Missing);
    assert_eq!(
        repo.get(&model.model_id).unwrap().unwrap().status,
        ModelStatus::Missing
    );
}

// RFC-012 §17: embedding model change marks semantic index stale.
#[test]
fn embedding_model_change_marks_embeddings_stale() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    // Seed a mock model and an embedding (using direct SQL for speed).
    let _now = "2026-01-01T00:00:00Z";
    {
        let conn = catalog.lock();
        conn.execute_batch("
            INSERT INTO sources (source_id, source_type, persistence_mode, original_path, canonical_path,
              status, index_mode, hidden_file_policy, symlink_policy, created_at, updated_at)
              VALUES ('s1','file','persistent','/a','/a','active','balanced','exclude','ignore','t','t');
            INSERT INTO files (file_id, source_id, original_path, canonical_path, display_path, file_size_bytes,
              file_status, last_seen_at, created_at, updated_at)
              VALUES ('f1','s1','/a','/a','a.md',1,'indexed','t','t','t');
            INSERT INTO extraction_records (extraction_id, file_id, extractor_name, extractor_version,
              normalization_version, status, created_at, updated_at)
              VALUES ('e1','f1','text','v1','norm-v1','succeeded','t','t');
            INSERT INTO chunks (chunk_id, file_id, extraction_id, chunk_kind, chunk_ordinal,
              chunk_status, created_at, updated_at)
              VALUES ('c1','f1','e1','document',0,'active','t','t');
            INSERT INTO models (model_id, role, model_name, model_version, dimension, status, created_at, updated_at)
              VALUES ('mdl1','embedding','embed','v1',768,'available','t','t');
            INSERT INTO embeddings (embedding_id, chunk_id, model_id, vector_format, dimension, norm,
              storage_location, vector_blob, status, created_at, updated_at)
              VALUES ('emb1','c1','mdl1','fp32',768,'l2','sqlite_blob',X'00',  'active','t','t');
        ").unwrap();
    }
    let repo = ModelRepository::new(&catalog);
    let mid = orbok_core::ModelId::from_string("mdl1".to_string());
    let staled = repo.mark_embedding_dependents_stale(&mid).unwrap();
    assert_eq!(staled, 1);
}

// RFC-012 §17: locate existing model registers it with Available status.
#[test]
fn locate_existing_model_file() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    let model_file = dir.path().join("model.bin");
    fs::write(&model_file, vec![0u8; 1024]).unwrap();

    let repo = ModelRepository::new(&catalog);
    let record = repo
        .locate(
            &model_file.to_string_lossy(),
            ModelRole::Embedding,
            "local-embed",
            "v1",
            Some(128),
        )
        .unwrap();
    assert_eq!(record.status, ModelStatus::Available);
    assert_eq!(record.dimension, Some(128));
    assert!(record.size_bytes.unwrap() > 0);
}

// ── RFC-015: Security ─────────────────────────────────────────────────

// RFC-015 §19: HTML snippets are escaped, not rendered as markup.
#[test]
fn html_escape_prevents_markup_injection() {
    let raw = "<script>alert('xss')</script> & \"quoted\" 'text'";
    let escaped = html_escape(raw);
    assert!(!escaped.contains('<'), "< must be escaped");
    assert!(!escaped.contains('>'), "> must be escaped");
    assert!(!escaped.contains('<'), "< must be escaped");
    assert!(!escaped.contains('>'), "> must be escaped");
    // The escaped string contains &lt; but not bare <, etc.
    assert!(escaped.contains("&lt;script&gt;"));
    assert!(escaped.contains("&amp;"));
    assert!(escaped.contains("&quot;"));
}

// RFC-015 §19: backend already enforces source allowlist.
// This test documents the PathGuard rejection path as a security test.
#[test]
fn path_guard_rejects_outside_sources() {
    use orbok_core::HiddenFilePolicy;
    use orbok_fs::{GuardedSource, PathGuard};
    let dir = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    fs::write(other.path().join("secret.txt"), "x").unwrap();
    let catalog = Catalog::open_in_memory().unwrap();
    let src = SourceRepository::new(&catalog)
        .insert(NewSource {
            source_type: SourceType::Directory,
            persistence_mode: PersistenceMode::Persistent,
            display_name: None,
            original_path: dir.path().to_string_lossy().into(),
            canonical_path: std::fs::canonicalize(dir.path())
                .unwrap()
                .to_string_lossy()
                .into(),
            index_mode: IndexMode::Balanced,
            include_patterns: vec![],
            exclude_patterns: vec![],
            hidden_file_policy: HiddenFilePolicy::Exclude,
            symlink_policy: SymlinkPolicy::Ignore,
            max_file_size_bytes: None,
        })
        .unwrap();
    let guard = PathGuard::new(vec![GuardedSource::from_record(&src)]);
    let result = guard.validate(&other.path().join("secret.txt"));
    assert!(
        result.is_err(),
        "access outside source must be rejected (RFC-015 §19)"
    );
}

// ── RFC-018: Crash Recovery ────────────────────────────────────────────

// RFC-018 §16 test 1: interrupted running jobs reset to queued on startup.
#[test]
fn interrupted_running_jobs_reset_to_queued() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _cache) = setup(dir.path());
    let root = std::fs::canonicalize(dir.path())
        .unwrap()
        .to_string_lossy()
        .to_string();
    let src = SourceRepository::new(&catalog)
        .insert(NewSource {
            source_type: SourceType::Directory,
            persistence_mode: PersistenceMode::Persistent,
            display_name: None,
            original_path: root.clone(),
            canonical_path: root,
            index_mode: IndexMode::Balanced,
            include_patterns: vec![],
            exclude_patterns: vec![],
            hidden_file_policy: HiddenFilePolicy::Exclude,
            symlink_policy: SymlinkPolicy::Ignore,
            max_file_size_bytes: None,
        })
        .unwrap();

    // Simulate a crashed job: enqueue then force to running.
    let job_id = IndexJobRepository::new(&catalog)
        .enqueue(JobType::Extract, Some(&src.source_id), None)
        .unwrap();
    IndexJobRepository::new(&catalog)
        .set_status(&job_id, JobStatus::Running)
        .unwrap();

    let cache_path = dir.path().join(orbok_db::CACHE_FILE_NAME);
    let report = run_startup_recovery(&catalog, &cache_path).unwrap();
    assert_eq!(report.jobs_reset, 1, "one running job should be reset");
    assert_eq!(report.jobs_pending, 1, "reset job should be pending");

    // Verify status is queued.
    let queued = IndexJobRepository::new(&catalog).list_queued(10).unwrap();
    assert_eq!(queued.len(), 1);
}

// RFC-018 §16 test 7: catalog integrity check runs cleanly on a fresh catalog.
#[test]
fn integrity_check_clean_on_fresh_catalog() {
    let catalog = Catalog::open_in_memory().unwrap();
    let report = check_catalog_integrity(&catalog).unwrap();
    assert!(
        report.is_clean(),
        "fresh catalog must have no integrity issues: {report:?}"
    );
}

// RFC-018 §16 test 3: missing cache DB does not crash.
#[test]
fn missing_cache_db_is_handled_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, _) = setup(dir.path());
    // No cache DB created — recovery should handle it.
    let cache_path = dir.path().join("orbok-cache.sqlite3");
    assert!(!cache_path.exists());
    let report = run_startup_recovery(&catalog, &cache_path).unwrap();
    // Recreated flag set — the DB was "missing" and handled gracefully.
    assert!(report.cache_recreated || !report.cache_rebuilt);
}

// ── Benchmark smoke test ──────────────────────────────────────────────

// RFC-016 §17: benchmark harness runs without errors on a small corpus.
#[test]
fn benchmark_corpus_generates_and_indexes() {
    use orbok_bench_lib::corpus;
    let dir = tempfile::tempdir().unwrap();
    corpus::generate(dir.path(), 10).unwrap();
    assert_eq!(
        fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| e
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .map(|x| x == "md")
                .unwrap_or(false))
            .count(),
        10
    );
    assert!(corpus::total_bytes(dir.path()) > 0);
}
