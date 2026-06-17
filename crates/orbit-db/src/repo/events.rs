//! Application event log repository (RFC-002 §7.13).
//!
//! Log hygiene contract (NFR-014, RFC-015): events must not contain
//! document body text. Callers pass short messages and optional
//! pre-redacted JSON details.

use crate::catalog::{Catalog, db_err};
use orbit_core::{EventId, OrbitResult, now_iso8601};
use rusqlite::params;

/// Event severity (matches the `app_events.severity` CHECK).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Debug => "debug",
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

pub struct EventRepository<'a> {
    catalog: &'a Catalog,
}

impl<'a> EventRepository<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Append an event. `redacted_details_json` must already be free of
    /// document contents.
    pub fn append(
        &self,
        event_type: &str,
        severity: Severity,
        message: &str,
        redacted_details_json: Option<&str>,
    ) -> OrbitResult<()> {
        let conn = self.catalog.lock();
        conn.execute(
            "INSERT INTO app_events (event_id, event_type, severity, message, \
             redacted_details_json, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                EventId::generate().as_str(),
                event_type,
                severity.as_str(),
                message,
                redacted_details_json,
                now_iso8601(),
            ],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// Most recent events, newest first.
    pub fn recent(&self, limit: u32) -> OrbitResult<Vec<(String, String, String)>> {
        let conn = self.catalog.lock();
        let mut stmt = conn
            .prepare(
                "SELECT event_type, severity, message FROM app_events \
                 ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(db_err)?);
        }
        Ok(out)
    }
}
