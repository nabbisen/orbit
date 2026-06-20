//! End-to-end cleanup service (M10, RFC-011 §11): combines catalog-side
//! cleanup (via [`CleanupExecutor`]) with cache-side cleanup (via
//! [`CacheService`]), driven by a validated [`CleanupPlan`].
//!
//! Call `CleanupService::run_safe` for ordinary cleanup; it will never
//! touch persistent source settings. For destructive operations use
//! `run_reset` with an explicit confirmation token.

use orbok_cache::CacheService;
use orbok_core::{CleanupAction, CleanupPlan, OrbokResult};
use orbok_db::Catalog;
use orbok_db::repo::CleanupExecutor;
use std::path::Path;
use tracing::info;

/// Combined cleanup outcome (catalog + cache sides).
#[derive(Debug, Default)]
pub struct FullCleanupOutcome {
    pub catalog_rows_deleted: u64,
    /// Approximate cache bytes freed (0 if cache cleanup is not applicable).
    pub cache_bytes_freed: u64,
}

/// Orchestrates catalog and cache cleanup (RFC-011 §8).
pub struct CleanupService<'a> {
    catalog: &'a Catalog,
    cache: &'a CacheService,
    cache_db_path: &'a Path,
}

impl<'a> CleanupService<'a> {
    pub fn new(catalog: &'a Catalog, cache: &'a CacheService, cache_db_path: &'a Path) -> Self {
        Self {
            catalog,
            cache,
            cache_db_path,
        }
    }

    /// Safe cleanup: validates the plan cannot touch persistent data, then
    /// runs catalog-side and cache-side operations atomically in intent
    /// (RFC-011 §8 "lifecycle-aware cleanup").
    pub fn run_safe(&self, plan: &CleanupPlan) -> OrbokResult<FullCleanupOutcome> {
        // Catalog side.
        let catalog_outcome = CleanupExecutor::new(self.catalog).run_safe(plan)?;
        info!(
            action = ?plan.action,
            rows = catalog_outcome.deleted_rows,
            "catalog cleanup completed"
        );

        // Cache side: map CleanupAction to cache namespace operations.
        let cache_bytes_freed = self.run_cache_side(plan)?;
        if cache_bytes_freed > 0 {
            info!(bytes = cache_bytes_freed, "cache cleanup freed space");
        }

        Ok(FullCleanupOutcome {
            catalog_rows_deleted: catalog_outcome.deleted_rows,
            cache_bytes_freed,
        })
    }

    /// Destructive catalog reset (requires confirmed ResetCatalog plan).
    pub fn run_reset(
        &self,
        plan: &CleanupPlan,
        keep_settings: bool,
    ) -> OrbokResult<FullCleanupOutcome> {
        // Catalog reset.
        let catalog_outcome =
            CleanupExecutor::new(self.catalog).run_reset_catalog(plan, keep_settings)?;

        // Purge all cache namespaces (RFC-011 §13: full reset clears caches).
        let cache_bytes_freed = self.purge_all_cache_namespaces()?;

        info!(
            rows = catalog_outcome.deleted_rows,
            cache_freed = cache_bytes_freed,
            "catalog reset completed"
        );

        Ok(FullCleanupOutcome {
            catalog_rows_deleted: catalog_outcome.deleted_rows,
            cache_bytes_freed,
        })
    }

    fn run_cache_side(&self, plan: &CleanupPlan) -> OrbokResult<u64> {
        use orbok_cache::{EngineOptions, OrbokCacheNamespace};

        let size_before = self.cache_db_path.metadata().map(|m| m.len()).unwrap_or(0);

        match plan.action {
            CleanupAction::ClearSnippetCache | CleanupAction::ClearExpiredSearchCache => {
                // Purge the preview-cache namespace.
                let engine = self.cache.engine::<Vec<u8>>(
                    self.catalog,
                    &OrbokCacheNamespace::PreviewCache,
                    EngineOptions::default(),
                )?;
                engine
                    .cleanup_expired()
                    .map_err(|e| orbok_core::OrbokError::Cache(e.to_string()))?;
                engine
                    .shrink_database()
                    .map_err(|e| orbok_core::OrbokError::Cache(e.to_string()))?;
            }
            CleanupAction::ClearTemporaryExtraction
            | CleanupAction::RemoveTemporarySourceIndexes => {
                // Purge extract-segments namespace.
                let engine = self.cache.engine::<Vec<u8>>(
                    self.catalog,
                    &OrbokCacheNamespace::ExtractSegments,
                    EngineOptions::default(),
                )?;
                engine
                    .purge_stale_versions()
                    .map_err(|e| orbok_core::OrbokError::Cache(e.to_string()))?;
                engine
                    .cleanup_missing_files()
                    .map_err(|e| orbok_core::OrbokError::Cache(e.to_string()))?;
            }
            CleanupAction::RemoveReplacedStaleIndexes => {
                // Clean up chunk and embedding bundle caches.
                for ns in [
                    OrbokCacheNamespace::ChunkBundle,
                    OrbokCacheNamespace::ExtractSegments,
                ] {
                    let engine = self.cache.engine::<Vec<u8>>(
                        self.catalog,
                        &ns,
                        EngineOptions::default(),
                    )?;
                    engine
                        .cleanup_missing_files()
                        .map_err(|e| orbok_core::OrbokError::Cache(e.to_string()))?;
                }
            }
            _ => {}
        }

        let size_after = self.cache_db_path.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(size_before.saturating_sub(size_after))
    }

    fn purge_all_cache_namespaces(&self) -> OrbokResult<u64> {
        use orbok_cache::{EngineOptions, OrbokCacheNamespace};
        let size_before = self.cache_db_path.metadata().map(|m| m.len()).unwrap_or(0);
        for ns in [
            OrbokCacheNamespace::ExtractSegments,
            OrbokCacheNamespace::ChunkBundle,
            OrbokCacheNamespace::PreviewCache,
        ] {
            let engine =
                self.cache
                    .engine::<Vec<u8>>(self.catalog, &ns, EngineOptions::default())?;
            let _ = engine.purge_stale_versions();
            let _ = engine.cleanup_expired();
            let _ = engine.shrink_database();
        }
        let size_after = self.cache_db_path.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(size_before.saturating_sub(size_after))
    }
}
