//! Cache service over localcache (Appendix A §8–§12).
//!
//! Rules enforced here:
//! - cache payloads live in `orbit-cache.sqlite3`, never in the catalog
//!   (Appendix A §3);
//! - the catalog stays authoritative — this service stores derived
//!   payloads only, keyed by canonical source path;
//! - reads and writes take a [`ValidatedPath`] so nothing outside the
//!   PathGuard boundary can be cached (RFC-003 §8 carried through);
//! - cleanup runs only from a validated [`CleanupPlan`] (RFC-001 §14);
//! - engines self-register in the catalog `cache_engines` table
//!   (RFC-002 §7.16) so the storage dashboard can enumerate them.

use crate::namespace::OrbitCacheNamespace;
use localcache::{CacheEngine, ChangeDetectionMode, LocalFileCacheError};
use orbit_core::{CleanupAction, CleanupPlan, OrbitError, OrbitResult};
use orbit_db::{CACHE_FILE_NAME, Catalog};
use orbit_fs::ValidatedPath;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Tuning for one engine.
#[derive(Debug, Clone, Default)]
pub struct EngineOptions {
    /// Time-to-live; `None` keeps entries until invalidated.
    pub ttl: Option<Duration>,
    /// LRU bound; `None` is unbounded (storage budget enforced via
    /// cleanup instead).
    pub max_entries: Option<usize>,
}

/// Result of a cache-side cleanup run.
#[derive(Debug, Clone, Default)]
pub struct CacheCleanupOutcome {
    pub removed_entries: u64,
}

/// Per-namespace usage for storage accounting (Appendix A §11).
#[derive(Debug, Clone)]
pub struct NamespaceUsage {
    pub namespace: String,
    pub entries: u64,
    pub payload_bytes: u64,
}

/// The orbit cache service. One per data directory.
pub struct CacheService {
    db_path: PathBuf,
}

impl CacheService {
    /// Create the service for a data directory; the payload database is
    /// `<data_dir>/orbit-cache.sqlite3` (Appendix A §3).
    pub fn new(data_dir: &Path) -> Self {
        Self {
            db_path: data_dir.join(CACHE_FILE_NAME),
        }
    }

    /// Payload database path (storage dashboard).
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Open a typed engine for `namespace`, registering it in the
    /// catalog `cache_engines` table. Change detection is
    /// metadata-then-full-hash (Appendix A §8: metadata fast path with
    /// hash confirmation, mirroring the scanner's policy).
    pub fn engine<T: Serialize + DeserializeOwned>(
        &self,
        catalog: &Catalog,
        namespace: &OrbitCacheNamespace,
        options: EngineOptions,
    ) -> OrbitResult<CacheEngine<T>> {
        let mut builder = CacheEngine::<T>::builder()
            .database(&self.db_path)
            .namespace(namespace.as_namespace())
            .payload_version(namespace.payload_version())
            .change_detection(ChangeDetectionMode::MetadataThenFullHash)
            .compress();
        builder = match options.ttl {
            Some(ttl) => builder.ttl(ttl),
            None => builder.no_ttl(),
        };
        if let Some(n) = options.max_entries {
            builder = builder.max_entries(n);
        }
        let engine = builder.build().map_err(cache_err)?;
        self.register_engine::<T>(catalog, namespace, &options)?;
        Ok(engine)
    }

    /// Freshness-checked read: returns the payload only when localcache
    /// confirms the source file is unchanged (Appendix A §8). The
    /// catalog/scanner remains the authority for file state.
    pub fn get_fresh<T: Serialize + DeserializeOwned>(
        engine: &CacheEngine<T>,
        path: &ValidatedPath,
    ) -> OrbitResult<Option<T>> {
        Ok(engine
            .get_if_fresh(&path.canonical)
            .map_err(cache_err)?
            .map(|entry| entry.payload))
    }

    /// Store a derived payload for a boundary-validated source path.
    pub fn put<T: Serialize + DeserializeOwned>(
        engine: &CacheEngine<T>,
        path: &ValidatedPath,
        payload: &T,
    ) -> OrbitResult<()> {
        engine.set(&path.canonical, payload).map_err(cache_err)
    }

    /// Invalidate one entry (e.g. file deleted from catalog).
    pub fn remove<T: Serialize + DeserializeOwned>(
        engine: &CacheEngine<T>,
        path: &ValidatedPath,
    ) -> OrbitResult<bool> {
        engine.remove(&path.canonical).map_err(cache_err)
    }

    /// Safe cleanup driven by a validated plan (RFC-001 §9, Appendix A
    /// §12). Maps each action to its payload namespaces and runs
    /// expiry + missing-file + stale-version maintenance there.
    pub fn run_safe_cleanup(
        &self,
        catalog: &Catalog,
        plan: &CleanupPlan,
    ) -> OrbitResult<CacheCleanupOutcome> {
        plan.assert_safe_for_ordinary_cleanup()?;
        let namespaces: Vec<OrbitCacheNamespace> = match plan.action {
            CleanupAction::ClearTemporaryExtraction => vec![OrbitCacheNamespace::ExtractSegments],
            CleanupAction::ClearSnippetCache => vec![OrbitCacheNamespace::PreviewCache],
            CleanupAction::RemoveReplacedStaleIndexes => vec![OrbitCacheNamespace::ChunkBundle],
            // Search cache lives in the catalog, not in localcache.
            CleanupAction::ClearExpiredSearchCache => vec![],
            _ => return Err(OrbitError::CleanupWouldTouchPersistentData),
        };
        let mut outcome = CacheCleanupOutcome::default();
        for namespace in namespaces {
            let engine = self.maintenance_engine(catalog, &namespace)?;
            outcome.removed_entries += engine.cleanup_expired().map_err(cache_err)? as u64;
            outcome.removed_entries += engine.cleanup_missing_files().map_err(cache_err)? as u64;
            outcome.removed_entries += engine.purge_stale_versions().map_err(cache_err)? as u64;
            tracing::debug!(
                namespace = namespace.as_namespace(),
                removed = outcome.removed_entries,
                "cache cleanup pass"
            );
        }
        Ok(outcome)
    }

    /// Reclaim file space after large deletions (storage dashboard's
    /// explicit "shrink" action; Appendix A §12).
    pub fn shrink(&self, catalog: &Catalog) -> OrbitResult<()> {
        let engine = self.maintenance_engine(catalog, &OrbitCacheNamespace::PreviewCache)?;
        engine.shrink_database().map_err(cache_err)
    }

    /// Usage per namespace for storage accounting (Appendix A §11).
    pub fn usage(
        &self,
        catalog: &Catalog,
        namespaces: &[OrbitCacheNamespace],
    ) -> OrbitResult<Vec<NamespaceUsage>> {
        let mut out = Vec::new();
        for namespace in namespaces {
            let engine = self.maintenance_engine(catalog, namespace)?;
            let stats = engine.cache_stats().map_err(cache_err)?;
            out.push(NamespaceUsage {
                namespace: stats.namespace,
                entries: stats.total_entries as u64,
                payload_bytes: stats.total_payload_bytes,
            });
        }
        Ok(out)
    }

    /// Untyped (JSON-payload) engine for maintenance operations that do
    /// not deserialize payloads.
    fn maintenance_engine(
        &self,
        catalog: &Catalog,
        namespace: &OrbitCacheNamespace,
    ) -> OrbitResult<CacheEngine<serde_json::Value>> {
        self.engine::<serde_json::Value>(catalog, namespace, EngineOptions::default())
    }

    /// Upsert the engine registration row (RFC-002 §7.16).
    fn register_engine<T>(
        &self,
        catalog: &Catalog,
        namespace: &OrbitCacheNamespace,
        options: &EngineOptions,
    ) -> OrbitResult<()> {
        let data_class = match namespace.data_class() {
            orbit_core::DataClass::RebuildableIndex => "rebuildable_index",
            _ => "ephemeral_cache",
        };
        let id = format!(
            "ce_{}",
            namespace.as_namespace().replace([':', '/'], "_")
        );
        let now = orbit_core::now_iso8601();
        let conn = catalog.lock();
        conn.execute(
            "INSERT INTO cache_engines (cache_engine_id, engine_kind, database_path, namespace, \
             data_class, payload_type, payload_version, ttl_seconds, max_entries, status, \
             created_at, updated_at) VALUES (?1,'localcache',?2,?3,?4,?5,?6,?7,?8,'active',?9,?9) \
             ON CONFLICT(engine_kind, database_path, namespace) DO UPDATE SET \
             payload_type = ?5, payload_version = ?6, ttl_seconds = ?7, max_entries = ?8, \
             status = 'active', updated_at = ?9",
            rusqlite::params![
                id,
                self.db_path.to_string_lossy(),
                namespace.as_namespace(),
                data_class,
                std::any::type_name::<T>(),
                namespace.payload_version(),
                options.ttl.map(|d| d.as_secs() as i64),
                options.max_entries.map(|n| n as i64),
                now,
            ],
        )
        .map_err(|e| OrbitError::Database(e.to_string()))?;
        Ok(())
    }
}

fn cache_err(e: LocalFileCacheError) -> OrbitError {
    OrbitError::Cache(e.to_string())
}
