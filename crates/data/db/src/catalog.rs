//! Catalog connection management (RFC-002 §5).
//!
//! The catalog is opened with foreign keys ON, WAL journal, NORMAL
//! synchronous, and in-memory temp store. Writes are serialized through
//! a single mutex-guarded connection (RFC-002 §5 "one serialized writer
//! path"); v1 keeps reads on the same connection for simplicity.

use crate::migrations;
use orbok_core::{OrbokError, OrbokResult};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// File name of the authoritative catalog database (Appendix A §3).
pub const CATALOG_FILE_NAME: &str = "orbok-catalog.sqlite3";

/// File name of the localcache-managed payload database (Appendix A §3).
pub const CACHE_FILE_NAME: &str = "orbok-cache.sqlite3";

/// The authoritative orbok catalog.
pub struct Catalog {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Catalog {
    /// Open (or create) the catalog at `path`, apply pragmas, and run
    /// pending migrations. Migration failure aborts startup (RFC-002
    /// §6.2).
    pub fn open(path: impl AsRef<Path>) -> OrbokResult<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path).map_err(db_err)?;
        Self::from_connection(conn, path)
    }

    /// Open an in-memory catalog (tests).
    pub fn open_in_memory() -> OrbokResult<Self> {
        let conn = Connection::open_in_memory().map_err(db_err)?;
        Self::from_connection(conn, PathBuf::from(":memory:"))
    }

    fn from_connection(conn: Connection, path: PathBuf) -> OrbokResult<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(db_err)?;
        // WAL is unsupported for in-memory databases; ignore that case.
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(db_err)?;
        conn.pragma_update(None, "temp_store", "MEMORY")
            .map_err(db_err)?;

        let catalog = Self {
            conn: Mutex::new(conn),
            path,
        };
        migrations::run_pending(&catalog)?;
        Ok(catalog)
    }

    /// Acquire the serialized connection. Repositories use this; the
    /// guard scope is kept short.
    pub fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .expect("catalog connection mutex poisoned — a repository panicked mid-write")
    }

    /// Path of the catalog database file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Current schema version (0 when no migration has been applied).
    pub fn schema_version(&self) -> OrbokResult<i64> {
        let conn = self.lock();
        let version = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        Ok(version)
    }
}

/// Map a rusqlite error to the typed orbok error.
pub(crate) fn db_err(e: rusqlite::Error) -> OrbokError {
    OrbokError::Database(e.to_string())
}
