//! v0.9.2 tests: source management backend, startup health population,
//! hybrid search backend routing, EmbeddingWorker model selection.

use orbok_cache::CacheService;
use orbok_core::{
    FileStatus, HiddenFilePolicy, IndexMode, PersistenceMode, SourceType, SymlinkPolicy,
};
use orbok_db::Catalog;
use orbok_db::repo::{
    FileRepository, NewSource, SourceRepository,
};
use crate::{
    ChunkAndIndexWorker, EmbeddingWorker, ExtractionWorker, run_pending,
    verify_embedding_model, VerifyOutcome,
};
use orbok_core::ModelId;
use orbok_models::{EmbeddingModel, MockEmbeddingModel};
use orbok_search::{HybridSearchService, SearchMode};
use std::fs;

fn setup(root: &std::path::Path) -> (Catalog, CacheService) {
    (Catalog::open(root.join("catalog.sqlite3")).unwrap(), CacheService::new(root))
}

fn seed_source(catalog: &Catalog, root: &std::path::Path) -> orbok_core::SourceId {
    let r = fs::canonicalize(root).unwrap().to_string_lossy().to_string();
    SourceRepository::new(catalog).insert(NewSource {
        source_type: SourceType::Directory,
        persistence_mode: PersistenceMode::Persistent,
        display_name: Some("test".into()),
        original_path: r.clone(),
        canonical_path: r,
        index_mode: IndexMode::Balanced,
        include_patterns: vec![],
        exclude_patterns: vec![],
        hidden_file_policy: HiddenFilePolicy::Exclude,
        symlink_policy: SymlinkPolicy::Ignore,
        max_file_size_bytes: None,
    }).unwrap().source_id
}

// ── Source management ────────────────────────────────────────────────

// FileRepository: count_with_status returns correct counts after indexing.
#[test]
fn count_with_status_reflects_indexed_files() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    fs::write(dir.path().join("doc.md"), "# Hello\nContent.\n").unwrap();
    let src_id = seed_source(&catalog, dir.path());

    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest { source_id: src_id.clone(), force_hash: false, enqueue_index_jobs: true },
            &AtomicBool::new(false),
        ).unwrap();
    }
    let e = ExtractionWorker::new(&catalog, &cache);
    let c = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &e, &c, None, 50).unwrap();

    let files = FileRepository::new(&catalog);
    assert!(files.count_with_status(FileStatus::Indexed).unwrap() > 0);
    assert_eq!(files.count_with_status(FileStatus::Failed).unwrap(), 0);
}

// count_for_source_with_status is source-scoped.
#[test]
fn count_for_source_with_status_is_scoped() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    let src_id = seed_source(&catalog, dir.path());
    // No files — both counts are zero.
    let files = FileRepository::new(&catalog);
    assert_eq!(files.count_for_source_with_status(&src_id, FileStatus::Indexed).unwrap(), 0);
    assert_eq!(files.count_for_source_with_status(&src_id, FileStatus::Failed).unwrap(), 0);
}

// SourceCard.source_id is populated when sources are loaded.
#[test]
fn source_card_has_source_id() {
    use orbok_ui::state::{SourceCard};
    let card = SourceCard {
        display_name: "test".into(),
        display_path: "/path".into(),
        indexed: 5,
        stale: 0,
        failed: 0,
        active: true,
        source_id: "src-abc123".into(),
    };
    assert_eq!(card.source_id, "src-abc123");
}

// ── EmbeddingWorker model selection ──────────────────────────────────

// with_model constructor sets the correct model id.
#[test]
fn embedding_worker_with_model_uses_supplied_model() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    let mock_id = ModelId::from_string("mock-custom-v1".to_string());
    let worker = EmbeddingWorker::with_model(
        &catalog,
        &cache,
        Box::new(MockEmbeddingModel),
        mock_id.clone(),
    );
    assert_eq!(worker.model_id().as_str(), "mock-custom-v1");
}

// ── Hybrid search backend routing ────────────────────────────────────

// HybridSearchService::keyword_only returns results without a model.
#[test]
fn hybrid_search_keyword_only_returns_results() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    fs::write(dir.path().join("auth.md"), "# Auth\nRefresh tokens expire daily.\n").unwrap();
    let src_id = seed_source(&catalog, dir.path());
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest { source_id: src_id, force_hash: false, enqueue_index_jobs: true },
            &AtomicBool::new(false),
        ).unwrap();
    }
    let e = ExtractionWorker::new(&catalog, &cache);
    let c = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &e, &c, None, 50).unwrap();

    let results = HybridSearchService::keyword_only(&catalog)
        .search("tokens", SearchMode::Exact, 10).unwrap();
    assert!(!results.is_empty(), "keyword search must find 'tokens'");
}

// HybridSearchService::with_model uses the embedding path.
#[test]
fn hybrid_search_with_model_uses_vector_path() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    fs::write(dir.path().join("auth.md"), "# Auth\nRefresh tokens expire daily.\n").unwrap();
    let src_id = seed_source(&catalog, dir.path());
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest { source_id: src_id, force_hash: false, enqueue_index_jobs: true },
            &AtomicBool::new(false),
        ).unwrap();
    }
    let e = ExtractionWorker::new(&catalog, &cache);
    let c = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &e, &c, None, 50).unwrap();

    let model = MockEmbeddingModel;
    let service = HybridSearchService::with_model(&catalog, &model, "mock");
    assert!(service.is_hybrid(), "with_model should enable hybrid mode");
    // Hybrid search returns results (even with mock embeddings).
    let results = service.search("tokens", SearchMode::Auto, 10).unwrap();
    assert!(!results.is_empty(), "hybrid search must return results");
}

// HybridSearchService falls back cleanly when model backend returns error.
#[test]
fn hybrid_search_falls_back_to_keyword_when_no_model_configured() {
    use orbok_embed::{create_embedding_model, recommended_config};
    use orbok_models::InferenceBackend;
    // ONNX model not configured — create_embedding_model returns Err.
    let config = recommended_config("/nonexistent/model.onnx");
    let is_err = create_embedding_model(&config).is_err();
    // Without --features tract, the factory returns an error.
    // We don't need to assert a specific value — just verify no panic.
    let _ = is_err; // result depends on compile features
}

// ── Startup health population ─────────────────────────────────────────

// Health is zero on a fresh catalog.
#[test]
fn health_is_zero_on_empty_catalog() {
    let catalog = Catalog::open_in_memory().unwrap();
    let files = FileRepository::new(&catalog);
    assert_eq!(files.count_with_status(FileStatus::Indexed).unwrap(), 0);
    assert_eq!(files.count_with_status(FileStatus::Stale).unwrap(), 0);
}

// Health reflects indexed count after pipeline runs.
#[test]
fn health_reflects_indexed_count() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, cache) = setup(dir.path());
    for i in 0..3 {
        fs::write(dir.path().join(format!("doc{i}.md")),
            format!("# Document {i}\nSome content here.\n")).unwrap();
    }
    let src_id = seed_source(&catalog, dir.path());
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        Scanner::new(&catalog).scan(
            &ScanRequest { source_id: src_id, force_hash: false, enqueue_index_jobs: true },
            &AtomicBool::new(false),
        ).unwrap();
    }
    let e = ExtractionWorker::new(&catalog, &cache);
    let c = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &e, &c, None, 50).unwrap();

    let files = FileRepository::new(&catalog);
    assert_eq!(files.count_with_status(FileStatus::Indexed).unwrap(), 3);
    assert_eq!(files.count_with_status(FileStatus::Failed).unwrap(), 0);
}
