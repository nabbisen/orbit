//! App settings repository (RFC-002 §7.1). Values are JSON; typed
//! accessors keep call sites honest. Settings are persistent catalog
//! data and survive every cleanup except an explicit full reset.

use crate::catalog::{Catalog, db_err};
use orbok_core::{OrbokError, OrbokResult, now_iso8601};
use rusqlite::params;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct SettingsRepository<'a> {
    catalog: &'a Catalog,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Store a typed setting under `key`.
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> OrbokResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| OrbokError::Database(format!("settings serialize: {e}")))?;
        let conn = self.catalog.lock();
        conn.execute(
            "INSERT INTO app_settings (key, value_json, updated_at) VALUES (?1, ?2, ?3) \
             ON CONFLICT(key) DO UPDATE SET value_json = ?2, updated_at = ?3",
            params![key, json, now_iso8601()],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// Read a typed setting; `None` when unset.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> OrbokResult<Option<T>> {
        let conn = self.catalog.lock();
        let json: Option<String> = conn
            .query_row(
                "SELECT value_json FROM app_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(db_err(other)),
            })?;
        match json {
            None => Ok(None),
            Some(json) => serde_json::from_str(&json)
                .map(Some)
                .map_err(|e| OrbokError::Database(format!("settings deserialize {key}: {e}"))),
        }
    }
}
