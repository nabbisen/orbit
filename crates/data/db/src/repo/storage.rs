//! Storage accounting repository (RFC-002 §7.12, RFC-001 §10).

use crate::catalog::{Catalog, db_err};
use orbok_core::{OrbokResult, StorageCategory, now_iso8601};
use rusqlite::params;

/// One accounting row per [`StorageCategory`].
#[derive(Debug, Clone)]
pub struct StorageRow {
    pub category: StorageCategory,
    pub size_bytes: u64,
    pub item_count: u64,
    pub updated_at: String,
}

pub struct StorageAccountingRepository<'a> {
    catalog: &'a Catalog,
}

impl<'a> StorageAccountingRepository<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Record (or refresh) the measured size of one category.
    pub fn upsert(
        &self,
        category: StorageCategory,
        size_bytes: u64,
        item_count: u64,
    ) -> OrbokResult<()> {
        let conn = self.catalog.lock();
        conn.execute(
            "INSERT INTO storage_accounting (category, size_bytes, item_count, updated_at) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(category) DO UPDATE SET size_bytes = ?2, item_count = ?3, updated_at = ?4",
            params![
                category.as_str(),
                size_bytes as i64,
                item_count as i64,
                now_iso8601()
            ],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// All recorded categories (Storage view breakdown).
    pub fn all(&self) -> OrbokResult<Vec<StorageRow>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare("SELECT category, size_bytes, item_count, updated_at FROM storage_accounting")
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (cat, size, count, updated) = row.map_err(db_err)?;
            // Unknown categories from future versions are skipped, not fatal.
            if let Some(category) = StorageCategory::ALL
                .iter()
                .find(|c| c.as_str() == cat)
                .copied()
            {
                out.push(StorageRow {
                    category,
                    size_bytes: size as u64,
                    item_count: count as u64,
                    updated_at: updated,
                });
            }
        }
        Ok(out)
    }
}
