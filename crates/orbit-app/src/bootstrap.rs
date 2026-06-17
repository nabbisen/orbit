//! Backend bootstrap: data-directory resolution, catalog open, and
//! initial view-model population. The `--check` mode runs this without
//! opening a window (useful on headless CI machines).

use orbit_db::{CATALOG_FILE_NAME, Catalog};
use orbit_db::repo::SettingsRepository;
use orbit_models::SearchCapability;
use orbit_ui::AppState;
use orbit_ui::i18n::Locale;
use std::path::PathBuf;

/// Resolve the orbit local-data directory.
/// Priority: `ORBIT_DATA_DIR` env var → platform-standard app-data dir.
pub fn data_dir() -> PathBuf {
    if let Ok(env) = std::env::var("ORBIT_DATA_DIR") {
        return PathBuf::from(env);
    }
    dirs::data_local_dir()
        .map(|d| d.join("orbit"))
        .unwrap_or_else(|| PathBuf::from("orbit-data"))
}

/// Open the catalog (creating the data dir if needed) and run pending
/// migrations. Returns an error if migration fails (RFC-002 §6.2:
/// startup aborts).
pub fn open_catalog(data_dir: &std::path::Path) -> orbit_core::OrbitResult<Catalog> {
    std::fs::create_dir_all(data_dir)?;
    Catalog::open(data_dir.join(CATALOG_FILE_NAME))
}

/// Load the initial view model from persisted settings. Falls back to
/// safe defaults when the catalog is empty or the setting is unset.
pub fn load_initial_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let dir = data_dir();
    let catalog = open_catalog(&dir)?;
    let settings = SettingsRepository::new(&catalog);

    let locale = settings
        .get::<String>("ui.locale")?
        .and_then(|s| Locale::parse(&s))
        .unwrap_or_default();

    let mut state = AppState::default();
    state.locale = locale;
    // Embedding model: absent until M7 (keyword-only mode is the v0.1 default).
    state.capability = SearchCapability::KeywordOnly;
    Ok(state)
}

/// Headless backend validation (CI / display-less machines).
///
/// Verifies: data-dir creation, catalog open, migration success,
/// schema version sanity. Exits 0 on success, non-zero on any error.
pub fn run_check() -> Result<(), Box<dyn std::error::Error>> {
    let dir = data_dir();
    tracing::info!(path = %dir.display(), "opening catalog");
    let catalog = open_catalog(&dir)?;
    let version = catalog.schema_version()?;
    let expected = orbit_db::migrations::latest_version();
    if version != expected {
        return Err(format!("schema version {version} != expected {expected}").into());
    }
    println!(
        "orbit --check OK  data_dir={}  schema_version={}",
        dir.display(),
        version
    );
    Ok(())
}
