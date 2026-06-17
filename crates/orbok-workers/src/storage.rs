//! Storage accounting (RFC-011 §9): measures actual orbok storage
//! consumption and updates the `storage_accounting` table.
//!
//! Measurements are approximate on purpose — exact byte-level
//! accounting per row is expensive; page-level and aggregate-query
//! measurements are fast and accurate enough for the Storage view.

use orbok_core::{OrbokError, OrbokResult, StorageCategory, now_iso8601};
use orbok_db::Catalog;
use orbok_db::repo::StorageAccountingRepository;
use rusqlite::params;
use std::path::Path;

/// Compute and persist storage accounting for the Storage view
/// (RFC-011 §9 "approximate by default").
///
/// Called by the worker pipeline after each indexing run and by the
/// Storage view's "refresh" action.
pub fn update_storage_accounting(
    catalog: &Catalog,
    cache_db_path: &Path,
) -> OrbokResult<Vec<(StorageCategory, u64, u64)>> {
    let storage = StorageAccountingRepository::new(catalog);
    let mut rows = Vec::new();

    macro_rules! measure {
        ($cat:expr, $size:expr, $count:expr) => {{
            storage.upsert($cat, $size, $count)?;
            rows.push(($cat, $size, $count));
        }};
    }

    let conn = catalog.lock();

    // Persistent catalog: approximate as the file size of catalog DB.
    // If in-memory (:memory:), report 0.
    let catalog_path = catalog.path();
    let catalog_bytes = if catalog_path.to_str() == Some(":memory:") {
        // Use page_count × page_size as proxy for in-memory databases.
        let pages: i64 = conn
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .unwrap_or(0);
        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |r| r.get(0))
            .unwrap_or(4096);
        (pages * page_size) as u64
    } else {
        std::fs::metadata(catalog_path).map(|m| m.len()).unwrap_or(0)
    };
    // Source count for "items"
    let source_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sources WHERE status != 'removed'", [], |r| r.get(0))
        .unwrap_or(0);
    drop(conn); // release before re-acquiring below
    measure!(StorageCategory::PersistentCatalog, catalog_bytes, source_count as u64);

    // Keyword index: row count from keyword_index_records.
    let conn = catalog.lock();
    let kw_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM keyword_index_records WHERE status='active'", [], |r| r.get(0))
        .unwrap_or(0);
    // Approximate size: 256 bytes per token record (FTS overhead).
    let kw_bytes = kw_count as u64 * 256;
    drop(conn);
    measure!(StorageCategory::KeywordIndex, kw_bytes, kw_count as u64);

    // Vector index: actual BLOB sizes.
    let conn = catalog.lock();
    let (emb_count, emb_bytes): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(LENGTH(vector_blob)), 0) FROM embeddings WHERE status='active'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));
    drop(conn);
    measure!(StorageCategory::VectorIndex, emb_bytes as u64, emb_count as u64);

    // Snippet cache: stored size_bytes column.
    let conn = catalog.lock();
    let (snip_count, snip_bytes): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0) FROM snippet_cache",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));
    drop(conn);
    measure!(StorageCategory::SnippetCache, snip_bytes as u64, snip_count as u64);

    // Search cache: row count (size unknown; estimate 512 bytes each).
    let conn = catalog.lock();
    let sr_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM search_result_cache", [], |r| r.get(0))
        .unwrap_or(0);
    drop(conn);
    measure!(StorageCategory::SearchCache, sr_count as u64 * 512, sr_count as u64);

    // Temporary extraction: localcache DB file size.
    let cache_bytes = std::fs::metadata(cache_db_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let conn = catalog.lock();
    let extract_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM extraction_records WHERE status='succeeded'", [], |r| r.get(0))
        .unwrap_or(0);
    drop(conn);
    measure!(StorageCategory::TemporaryExtraction, cache_bytes, extract_count as u64);

    // Logs: app_events row estimate.
    let conn = catalog.lock();
    let evt_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM app_events", [], |r| r.get(0))
        .unwrap_or(0);
    drop(conn);
    measure!(StorageCategory::Logs, evt_count as u64 * 256, evt_count as u64);

    // Model files: not tracked in v0.4 (full workflow lands in M12).
    measure!(StorageCategory::ModelFiles, 0, 0);

    Ok(rows)
}
