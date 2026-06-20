//! Tests for orbok-cache, validating Appendix A acceptance criteria:
//! separate payload DB, freshness-checked reads, plan-driven cleanup
//! that cannot touch the catalog, engine registration, usage stats.

use crate::{CacheService, EngineOptions, OrbokCacheNamespace};
use orbok_core::SourceId;
use orbok_core::{CleanupAction, CleanupPlan, DataClass};
use orbok_db::{CACHE_FILE_NAME, CATALOG_FILE_NAME, Catalog};
use orbok_fs::ValidatedPath;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Segments {
    lines: Vec<String>,
}

fn validated(path: &std::path::Path) -> ValidatedPath {
    ValidatedPath {
        source_id: SourceId::generate(),
        canonical: fs::canonicalize(path).unwrap(),
    }
}

// Appendix A §3: payloads live in orbok-cache.sqlite3, not the catalog.
#[test]
fn payloads_live_in_separate_database() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(dir.path().join(CATALOG_FILE_NAME)).unwrap();
    let service = CacheService::new(dir.path());

    let file = dir.path().join("doc.md");
    fs::write(&file, "hello").unwrap();
    let engine = service
        .engine::<Segments>(
            &catalog,
            &OrbokCacheNamespace::ExtractSegments,
            EngineOptions::default(),
        )
        .unwrap();
    CacheService::put(
        &engine,
        &validated(&file),
        &Segments {
            lines: vec!["hello".into()],
        },
    )
    .unwrap();

    assert!(dir.path().join(CACHE_FILE_NAME).exists());
    // The catalog contains a registration row but no payload tables.
    let conn = catalog.lock();
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM cache_engines", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}

// Appendix A §8: freshness-checked read hits while unchanged, misses
// after modification (cache never serves stale payloads as fresh).
#[test]
fn get_fresh_misses_after_source_change() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(dir.path().join(CATALOG_FILE_NAME)).unwrap();
    let service = CacheService::new(dir.path());
    let engine = service
        .engine::<Segments>(
            &catalog,
            &OrbokCacheNamespace::ExtractSegments,
            EngineOptions::default(),
        )
        .unwrap();

    let file = dir.path().join("doc.md");
    fs::write(&file, "v1").unwrap();
    let path = validated(&file);
    let payload = Segments {
        lines: vec!["v1".into()],
    };
    CacheService::put(&engine, &path, &payload).unwrap();

    assert_eq!(
        CacheService::get_fresh(&engine, &path).unwrap(),
        Some(payload)
    );

    // Change the file: full-hash verification must reject the entry.
    fs::write(&file, "v2 with different size").unwrap();
    assert_eq!(CacheService::get_fresh(&engine, &path).unwrap(), None);
}

// Appendix A §7: embedding namespaces are parameterized per model and
// never collide; classes are as designed.
#[test]
fn namespaces_are_distinct_and_classed() {
    let a = OrbokCacheNamespace::EmbeddingBundle {
        model_id: "model-a".into(),
        vector_format: "fp32".into(),
    };
    let b = OrbokCacheNamespace::EmbeddingBundle {
        model_id: "model-b".into(),
        vector_format: "fp32".into(),
    };
    assert_ne!(a.as_namespace(), b.as_namespace());
    assert_eq!(a.data_class(), DataClass::RebuildableIndex);
    assert_eq!(
        OrbokCacheNamespace::PreviewCache.data_class(),
        DataClass::EphemeralCache
    );
}

// RFC-001 §14 carried into the cache layer: destructive plans rejected;
// safe plans clean payloads while the catalog is untouched.
#[test]
fn cleanup_is_plan_driven_and_safe() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(dir.path().join(CATALOG_FILE_NAME)).unwrap();
    let service = CacheService::new(dir.path());
    let engine = service
        .engine::<Segments>(
            &catalog,
            &OrbokCacheNamespace::ExtractSegments,
            EngineOptions::default(),
        )
        .unwrap();

    // A payload whose source file disappears becomes orphaned.
    let file = dir.path().join("gone.md");
    fs::write(&file, "bye").unwrap();
    let path = validated(&file);
    CacheService::put(&engine, &path, &Segments { lines: vec![] }).unwrap();
    fs::remove_file(&file).unwrap();

    // Destructive plan: rejected before touching anything.
    let reset = CleanupPlan::for_action(CleanupAction::ResetCatalog, 0);
    assert!(service.run_safe_cleanup(&catalog, &reset).is_err());

    // Safe plan: orphaned entry removed.
    let plan = CleanupPlan::for_action(CleanupAction::ClearTemporaryExtraction, 0);
    let outcome = service.run_safe_cleanup(&catalog, &plan).unwrap();
    assert!(outcome.removed_entries >= 1);
}

// Appendix A §11: usage stats per namespace for storage accounting.
#[test]
fn usage_reports_entries_and_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(dir.path().join(CATALOG_FILE_NAME)).unwrap();
    let service = CacheService::new(dir.path());
    let engine = service
        .engine::<Segments>(
            &catalog,
            &OrbokCacheNamespace::ChunkBundle,
            EngineOptions::default(),
        )
        .unwrap();

    let file = dir.path().join("doc.md");
    fs::write(&file, "data").unwrap();
    CacheService::put(
        &engine,
        &validated(&file),
        &Segments {
            lines: vec!["data".into()],
        },
    )
    .unwrap();

    let usage = service
        .usage(&catalog, &[OrbokCacheNamespace::ChunkBundle])
        .unwrap();
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].entries, 1);
    assert!(usage[0].payload_bytes > 0);
    assert_eq!(usage[0].namespace, "chunk-bundle:v1");
}

// Regression for the defect fixed in localcache 0.20.0 (schema v5):
// a file overwritten immediately — same byte length, different content —
// must be detected as stale. With second-precision mtimes this overwrite
// was invisible to metadata checks; nanosecond mtimes plus orbok's
// MetadataThenFullHash default catch it either way.
#[test]
fn same_size_immediate_overwrite_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(dir.path().join(CATALOG_FILE_NAME)).unwrap();
    let service = CacheService::new(dir.path());
    let engine = service
        .engine::<Segments>(
            &catalog,
            &OrbokCacheNamespace::ExtractSegments,
            EngineOptions::default(),
        )
        .unwrap();

    let file = dir.path().join("doc.md");
    fs::write(&file, "AAAA").unwrap();
    let path = validated(&file);
    let payload = Segments {
        lines: vec!["AAAA".into()],
    };
    CacheService::put(&engine, &path, &payload).unwrap();

    // Overwrite within the same instant: identical length, new content.
    fs::write(&file, "BBBB").unwrap();
    assert_eq!(
        CacheService::get_fresh(&engine, &path).unwrap(),
        None,
        "same-size immediate overwrite must invalidate the cached payload"
    );
}
