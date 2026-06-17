//! Backend bootstrap: data-directory resolution, catalog open, initial
//! view-model population, and worker pipeline execution.

use orbok_core::OrbokResult;
use orbok_db::{CATALOG_FILE_NAME, Catalog};
use orbok_db::repo::SettingsRepository;
use orbok_models::SearchCapability;
use orbok_search::SearchService;
use orbok_ui::AppState;
use orbok_ui::i18n::Locale;
use std::path::PathBuf;

/// Resolve the orbok local-data directory.
pub fn data_dir() -> PathBuf {
    if let Ok(env) = std::env::var("ORBOK_DATA_DIR") {
        return PathBuf::from(env);
    }
    dirs::data_local_dir()
        .map(|d| d.join("orbok"))
        .unwrap_or_else(|| PathBuf::from("orbok-data"))
}

/// Open the catalog (creating the data dir if needed).
pub fn open_catalog(data_dir: &std::path::Path) -> OrbokResult<Catalog> {
    std::fs::create_dir_all(data_dir)?;
    Catalog::open(data_dir.join(CATALOG_FILE_NAME))
}

/// Build the initial app state from persisted settings.
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
    state.capability = SearchCapability::KeywordOnly;
    Ok(state)
}

/// Execute a keyword search and convert results to UI display structs.
pub(crate) fn run_search(
    catalog: &Catalog,
    query: &str,
    limit: u32,
) -> Result<Vec<orbok_ui::state::SearchResultDisplay>, Box<dyn std::error::Error>> {
    let service = SearchService::new(catalog);
    let results = service.search(query, limit)?;
    Ok(results
        .into_iter()
        .map(|r| orbok_ui::state::SearchResultDisplay {
            display_path: r.display_path,
            title: r.title,
            heading_path: r.heading_path,
            snippet: r.snippet,
            keyword_rank: r.keyword_rank,
            badges: r.badges.iter().map(|b| format!("{b:?}")).collect(),
        })
        .collect())
}

/// Headless backend validation (`--check` mode).
pub fn run_check() -> Result<(), Box<dyn std::error::Error>> {
    let dir = data_dir();
    tracing::info!(path = %dir.display(), "opening catalog");
    let catalog = open_catalog(&dir)?;
    let version = catalog.schema_version()?;
    let expected = orbok_db::migrations::latest_version();
    if version != expected {
        return Err(format!("schema version {version} != expected {expected}").into());
    }
    println!(
        "orbok --check OK  data_dir={}  schema_version={}",
        dir.display(),
        version
    );
    Ok(())
}

/// Persist locale setting to the catalog when the user changes it.
pub fn persist_locale(catalog: &Catalog, locale: &orbok_ui::i18n::Locale) -> OrbokResult<()> {
    orbok_db::repo::SettingsRepository::new(catalog)
        .set("ui.locale", &locale.as_str().to_string())
}
