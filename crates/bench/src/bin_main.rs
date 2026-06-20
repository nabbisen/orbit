//! orbok benchmark harness (RFC-016).
//!
//! Measures:
//! - Indexing throughput (files/s, MB/s)
//! - Search latency (p50/p95/p99 ms)
//! - Retrieval quality: recall@k against labeled queries
//! - Storage growth (bytes per indexed file)
//!
//! Design rules (RFC-016 §18):
//! - No benchmark uploads documents.
//! - Document contents are not logged by default.
//! - Output is JSON + Markdown.

mod corpus;
mod metrics;
mod queries;
mod report;

use orbok_cache::CacheService;
use orbok_db::Catalog;
use orbok_workers::{ChunkAndIndexWorker, ExtractionWorker, run_pending};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("warn").init();

    let args: Vec<String> = std::env::args().collect();
    let n_docs: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);
    let out_dir = PathBuf::from(
        args.get(2)
            .map(|s| s.as_str())
            .unwrap_or("orbok-bench-output"),
    );
    std::fs::create_dir_all(&out_dir)?;

    eprintln!("orbok-bench: generating {n_docs} synthetic documents…");
    let work_dir = tempfile::tempdir()?;
    corpus::generate(work_dir.path(), n_docs)?;

    eprintln!("orbok-bench: opening catalog…");
    let catalog = Catalog::open(work_dir.path().join("bench-catalog.sqlite3"))?;
    let cache = CacheService::new(work_dir.path());

    // Register one source for the synthetic corpus.
    {
        use orbok_core::{HiddenFilePolicy, IndexMode, PersistenceMode, SourceType, SymlinkPolicy};
        use orbok_db::repo::{NewSource, SourceRepository};
        let root = std::fs::canonicalize(work_dir.path())?
            .to_string_lossy()
            .to_string();
        SourceRepository::new(&catalog).insert(NewSource {
            source_type: SourceType::Directory,
            persistence_mode: PersistenceMode::Persistent,
            display_name: Some("bench-corpus".into()),
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

    // Scan → queue extract jobs.
    {
        use orbok_fs::{ScanRequest, Scanner};
        use std::sync::atomic::AtomicBool;
        let sources = orbok_db::repo::SourceRepository::new(&catalog);
        let src = sources.list_active()?;
        if let Some(source) = src.first() {
            let scanner = Scanner::new(&catalog);
            scanner.scan(
                &ScanRequest {
                    source_id: source.source_id.clone(),
                    force_hash: false,
                    enqueue_index_jobs: true,
                },
                &AtomicBool::new(false),
            )?;
        }
    }

    eprintln!("orbok-bench: indexing {n_docs} documents…");
    let index_start = Instant::now();
    let extract = ExtractionWorker::new(&catalog, &cache);
    let chunk = ChunkAndIndexWorker::new(&catalog, &cache);
    run_pending(&catalog, &extract, &chunk, None, n_docs as u32 * 4)?;
    let index_elapsed_ms = index_start.elapsed().as_millis() as u64;

    // Measure catalog size.
    let catalog_size = std::fs::metadata(work_dir.path().join("bench-catalog.sqlite3"))
        .map(|m| m.len())
        .unwrap_or(0);
    let corpus_size: u64 = corpus::total_bytes(work_dir.path());

    eprintln!("orbok-bench: running search latency benchmark…");
    let latencies = metrics::measure_search_latency(&catalog, queries::LABELED_QUERIES)?;

    eprintln!("orbok-bench: computing recall@k…");
    let recall = metrics::compute_recall(&catalog, queries::LABELED_QUERIES)?;

    let result = report::BenchmarkResult {
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
    };

    result.write_json(&out_dir.join("orbok-bench-results.json"))?;
    result.write_markdown(&out_dir.join("orbok-bench-report.md"))?;
    eprintln!("orbok-bench: results written to {}", out_dir.display());
    result.print_summary();
    Ok(())
}
