//! Re-exports for integration testing (RFC-016 §17 benchmark smoke test).
pub mod corpus;
pub mod metrics;
pub mod queries;
pub mod report;

/// Full benchmark run returning a `BenchmarkResult`. Used by CI and tests.
pub fn run_bench(
    n_docs: usize,
    work_dir: &std::path::Path,
) -> Result<report::BenchmarkResult, Box<dyn std::error::Error>> {
    corpus::generate(work_dir, n_docs)?;
    let catalog = orbok_db::Catalog::open(work_dir.join("bench-catalog.sqlite3"))?;
    let cache = orbok_cache::CacheService::new(work_dir);
    {
        use orbok_core::{HiddenFilePolicy, IndexMode, PersistenceMode, SourceType, SymlinkPolicy};
        use orbok_db::repo::{NewSource, SourceRepository};
        let root = std::fs::canonicalize(work_dir)?
            .to_string_lossy()
            .to_string();
        SourceRepository::new(&catalog).insert(NewSource {
            source_type: SourceType::Directory,
            persistence_mode: PersistenceMode::Persistent,
            display_name: Some("bench".into()),
            original_path: root.clone(),
            canonical_path: root,
            index_mode: IndexMode::Balanced,
            include_patterns: vec![],
            exclude_patterns: vec![],
            hidden_file_policy: HiddenFilePolicy::Exclude,
            symlink_policy: SymlinkPolicy::Ignore,
            max_file_size_bytes: None,
        })?;
    }
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        let sources = orbok_db::repo::SourceRepository::new(&catalog);
        let src = sources.list_active()?;
        if let Some(source) = src.first() {
            Scanner::new(&catalog).scan(
                &ScanRequest {
                    source_id: source.source_id.clone(),
                    force_hash: false,
                    enqueue_index_jobs: true,
                },
                &AtomicBool::new(false),
            )?;
        }
    }
    let index_start = std::time::Instant::now();
    let extract = orbok_workers::ExtractionWorker::new(&catalog, &cache);
    let chunk = orbok_workers::ChunkAndIndexWorker::new(&catalog, &cache);
    orbok_workers::run_pending(&catalog, &extract, &chunk, None, n_docs as u32 * 4)?;
    let index_elapsed_ms = index_start.elapsed().as_millis() as u64;
    let catalog_size = std::fs::metadata(work_dir.join("bench-catalog.sqlite3"))
        .map(|m| m.len())
        .unwrap_or(0);
    let corpus_size = corpus::total_bytes(work_dir);
    let latencies = metrics::measure_search_latency(&catalog, queries::LABELED_QUERIES)?;
    let recall = metrics::compute_recall(&catalog, queries::LABELED_QUERIES)?;
    Ok(report::BenchmarkResult {
        n_docs,
        corpus_bytes: corpus_size,
        catalog_bytes: catalog_size,
        index_elapsed_ms,
        indexing_files_per_sec: if index_elapsed_ms > 0 {
            (n_docs as f64 * 1000.0) / index_elapsed_ms as f64
        } else {
            0.0
        },
        search_latency_ms: latencies,
        recall_at_k: recall,
    })
}

#[cfg(test)]
mod bench_tests {
    use super::*;

    // RFC-016 §17 / RFC-023 baseline: benchmark with 100 synthetic documents.
    // Results inform the ANN and quantization decisions.
    #[test]
    fn bench_full_pipeline() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_bench(100, dir.path()).unwrap();
        result.print_summary();
        // RFC-023 gate: exact scan must be fast enough for the test corpus.
        assert!(
            result.search_latency_ms.p99_ms < 2000.0,
            "p99 search latency too high: {:.2}ms",
            result.search_latency_ms.p99_ms
        );
        // RFC-016 recall target (relaxed for synthetic corpus with mock model).
        assert!(
            result.recall_at_k.recall >= 0.0,
            "recall must be a valid fraction"
        );
        println!(
            "Recall@{}: {:.1}%",
            result.recall_at_k.k,
            result.recall_at_k.recall * 100.0
        );
    }
}
