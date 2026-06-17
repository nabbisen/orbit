//! # orbit-db
//!
//! The authoritative orbit SQLite catalog (RFC-002): connection
//! management, append-only migrations, and the repository layer.
//!
//! Design rules implemented here:
//! - the catalog is authoritative; localcache payloads are elsewhere
//!   (Appendix A §3, separate `orbit-cache.sqlite3`);
//! - SQL stays inside repositories — application code sees typed records
//!   (RFC-002 §8);
//! - cleanup is executed only from a validated [`orbit_core::CleanupPlan`]
//!   (RFC-001 §14).

pub mod catalog;
pub mod migrations;
pub mod repo;

#[cfg(test)]
mod tests;

pub use catalog::{CACHE_FILE_NAME, CATALOG_FILE_NAME, Catalog};
