//! File catalog repository (RFC-002 §7.3, RFC-004).
//!
//! The scanner drives these operations: upsert on discovery, metadata
//! comparison for change detection, missing-marking for unseen files.
//! File catalog records are persistent catalog data (RFC-001 §5.3) and
//! survive index cleanup.

use crate::catalog::{Catalog, db_err};
use orbok_core::{FileId, FileStatus, OrbokError, OrbokResult, SourceId, now_iso8601};
use rusqlite::{Row, params};

/// A cataloged file.
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub file_id: FileId,
    pub source_id: SourceId,
    pub original_path: String,
    pub canonical_path: String,
    pub display_path: String,
    pub extension: Option<String>,
    pub file_size_bytes: u64,
    pub modified_at: Option<String>,
    pub platform_file_key: Option<String>,
    pub content_hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub file_status: FileStatus,
    pub last_seen_at: String,
    pub last_indexed_at: Option<String>,
}

/// Parameters for inserting a newly discovered file.
#[derive(Debug, Clone)]
pub struct NewFile {
    pub source_id: SourceId,
    pub original_path: String,
    pub canonical_path: String,
    pub display_path: String,
    pub extension: Option<String>,
    pub metadata: ObservedMetadata,
    pub status: FileStatus,
}

/// Metadata observed on disk during a scan (RFC-004 §9.1 fast check).
#[derive(Debug, Clone, Default)]
pub struct ObservedMetadata {
    pub file_size_bytes: u64,
    pub modified_at: Option<String>,
    pub platform_file_key: Option<String>,
    pub content_hash: Option<String>,
}

const COLUMNS: &str = "file_id, source_id, original_path, canonical_path, display_path, \
     extension, file_size_bytes, modified_at, platform_file_key, content_hash, hash_algorithm, \
     file_status, last_seen_at, last_indexed_at";

/// Repository over the `files` table.
pub struct FileRepository<'a> {
    catalog: &'a Catalog,
}

impl<'a> FileRepository<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Look up a file by its identity key (source, canonical path).
    pub fn get_by_path(
        &self,
        source_id: &SourceId,
        canonical_path: &str,
    ) -> OrbokResult<Option<FileRecord>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM files WHERE source_id = ?1 AND canonical_path = ?2"
            ))
            .map_err(db_err)?;
        let mut rows = stmt
            .query_map(params![source_id.as_str(), canonical_path], row_to_record)
            .map_err(db_err)?;
        match rows.next() {
            Some(r) => Ok(Some(r.map_err(db_err)??)),
            None => Ok(None),
        }
    }

    /// Insert a newly discovered file.
    pub fn insert(&self, new: NewFile) -> OrbokResult<FileRecord> {
        let id = FileId::generate();
        let now = now_iso8601();
        let hash_algorithm = new.metadata.content_hash.as_ref().map(|_| "sha256");
        let conn = self.catalog.lock();
        conn.execute(
            "INSERT INTO files (file_id, source_id, original_path, canonical_path, display_path, \
             extension, file_size_bytes, modified_at, platform_file_key, content_hash, \
             hash_algorithm, file_status, last_seen_at, last_scanned_at, created_at, updated_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13,?13,?13)",
            params![
                id.as_str(),
                new.source_id.as_str(),
                new.original_path,
                new.canonical_path,
                new.display_path,
                new.extension,
                new.metadata.file_size_bytes as i64,
                new.metadata.modified_at,
                new.metadata.platform_file_key,
                new.metadata.content_hash,
                hash_algorithm,
                new.status.as_str(),
                now,
            ],
        )
        .map_err(db_err)?;
        drop(conn);
        self.get_by_path_id(&id)
    }

    fn get_by_path_id(&self, id: &FileId) -> OrbokResult<FileRecord> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare(&format!("SELECT {COLUMNS} FROM files WHERE file_id = ?1"))
            .map_err(db_err)?;
        let mut rows = stmt
            .query_map(params![id.as_str()], row_to_record)
            .map_err(db_err)?;
        match rows.next() {
            Some(r) => r.map_err(db_err)?,
            None => Err(OrbokError::FileNotFound),
        }
    }

    /// Touch a file confirmed unchanged by the metadata check.
    pub fn touch_seen(&self, id: &FileId) -> OrbokResult<()> {
        let now = now_iso8601();
        let conn = self.catalog.lock();
        conn.execute(
            "UPDATE files SET last_seen_at = ?2, last_scanned_at = ?2, updated_at = ?2 \
             WHERE file_id = ?1",
            params![id.as_str(), now],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// Record changed on-disk metadata and the resulting status
    /// transition (RFC-004 §12 stale detection).
    pub fn update_observed(
        &self,
        id: &FileId,
        metadata: &ObservedMetadata,
        status: FileStatus,
    ) -> OrbokResult<()> {
        let now = now_iso8601();
        let hash_algorithm = metadata.content_hash.as_ref().map(|_| "sha256");
        let conn = self.catalog.lock();
        conn.execute(
            "UPDATE files SET file_size_bytes = ?2, modified_at = ?3, platform_file_key = ?4, \
             content_hash = COALESCE(?5, content_hash), \
             hash_algorithm = COALESCE(?6, hash_algorithm), file_status = ?7, \
             last_seen_at = ?8, last_scanned_at = ?8, updated_at = ?8 WHERE file_id = ?1",
            params![
                id.as_str(),
                metadata.file_size_bytes as i64,
                metadata.modified_at,
                metadata.platform_file_key,
                metadata.content_hash,
                hash_algorithm,
                status.as_str(),
                now,
            ],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// Set status only (e.g. permission_denied observed mid-scan).
    pub fn set_status(&self, id: &FileId, status: FileStatus) -> OrbokResult<()> {
        let conn = self.catalog.lock();
        conn.execute(
            "UPDATE files SET file_status = ?2, updated_at = ?3 WHERE file_id = ?1",
            params![id.as_str(), status.as_str(), now_iso8601()],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// RFC-004 §11: mark files of `source_id` not seen since `cutoff`
    /// as Missing — never Deleted (drives may be disconnected). Returns
    /// the number of newly missing files.
    pub fn mark_missing_unseen(&self, source_id: &SourceId, cutoff: &str) -> OrbokResult<u64> {
        let conn = self.catalog.lock();
        let n = conn
            .execute(
                "UPDATE files SET file_status = 'missing', updated_at = ?3 \
                 WHERE source_id = ?1 AND last_seen_at < ?2 \
                 AND file_status NOT IN ('missing', 'deleted')",
                params![source_id.as_str(), cutoff, now_iso8601()],
            )
            .map_err(db_err)?;
        Ok(n as u64)
    }

    /// Status counts for one source (Indexing/Sources view summaries).
    pub fn count_by_status(&self, source_id: &SourceId) -> OrbokResult<Vec<(FileStatus, u64)>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare(
                "SELECT file_status, COUNT(*) FROM files WHERE source_id = ?1 GROUP BY file_status",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map(params![source_id.as_str()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (status, count) = row.map_err(db_err)?;
            out.push((FileStatus::parse(&status)?, count as u64));
        }
        Ok(out)
    }
}

fn row_to_record(row: &Row<'_>) -> rusqlite::Result<OrbokResult<FileRecord>> {
    let status: String = row.get(11)?;
    let size: i64 = row.get(6)?;
    Ok((|| {
        Ok(FileRecord {
            file_id: FileId::from_string(row.get::<_, String>(0).map_err(db_err)?),
            source_id: SourceId::from_string(row.get::<_, String>(1).map_err(db_err)?),
            original_path: row.get(2).map_err(db_err)?,
            canonical_path: row.get(3).map_err(db_err)?,
            display_path: row.get(4).map_err(db_err)?,
            extension: row.get(5).map_err(db_err)?,
            file_size_bytes: size as u64,
            modified_at: row.get(7).map_err(db_err)?,
            platform_file_key: row.get(8).map_err(db_err)?,
            content_hash: row.get(9).map_err(db_err)?,
            hash_algorithm: row.get(10).map_err(db_err)?,
            file_status: FileStatus::parse(&status)?,
            last_seen_at: row.get(12).map_err(db_err)?,
            last_indexed_at: row.get(13).map_err(db_err)?,
        })
    })())
}
