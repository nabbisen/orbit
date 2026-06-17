//! Backend bootstrap: data-directory resolution, catalog open, settings
//! load, model verification, and initial view-model population.
//!
//! Startup sequence (RFC-027, design §startup):
//! 1. resolve data directory (env > portable flag > platform dir)
//! 2. open catalog and run migrations
//! 3. run startup recovery (RFC-018)
//! 4. load `OrbokSettings` from platform config dir
//! 5. verify embedding model files (design §startup-verify)
//! 6. build initial `AppState` (wizard active if model missing)

use orbok_core::OrbokResult;
use orbok_db::{CATALOG_FILE_NAME, Catalog};
use orbok_db::repo::SettingsRepository;
use orbok_models::SearchCapability;
use orbok_search::SearchService;
use orbok_ui::AppState;
use orbok_ui::i18n::Locale;
use orbok_ui::state::{WizardFileCheck, WizardState};
use orbok_workers::{VerifyOutcome, verify_embedding_model};
use std::path::PathBuf;

use crate::settings::{OrbokSettings, load_settings};

/// Resolve the orbok local-data directory.
pub fn data_dir() -> PathBuf {
    if let Ok(env) = std::env::var("ORBOK_DATA_DIR") {
        return PathBuf::from(env);
    }
    dirs::data_local_dir()
        .map(|d| d.join("orbok"))
        .unwrap_or_else(|| PathBuf::from("orbok-data"))
}

/// Resolve considering `--portable` flag (RFC-030).
pub fn data_dir_for_args(portable: bool) -> PathBuf {
    if portable { PathBuf::from("orbok-data") } else { data_dir() }
}

/// Open the catalog, creating the data directory if needed.
pub fn open_catalog(data_dir: &std::path::Path) -> OrbokResult<Catalog> {
    std::fs::create_dir_all(data_dir)?;
    Catalog::open(data_dir.join(CATALOG_FILE_NAME))
}

/// Build the initial `AppState` from persisted settings and startup
/// model verification. Activates the wizard when any required model
/// file is missing or not yet configured.
pub fn load_initial_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let dir = data_dir();
    let catalog = open_catalog(&dir)?;

    // RFC-018: reset any jobs left running from a crashed session.
    let cache_path = dir.join(orbok_db::CACHE_FILE_NAME);
    let recovery = orbok_workers::run_startup_recovery(&catalog, &cache_path)?;
    if recovery.jobs_reset > 0 {
        tracing::warn!(reset = recovery.jobs_reset, "reset interrupted jobs on startup");
    }

    // Load persisted OrbokSettings (app-json-settings).
    let settings = load_settings();

    // Locale: prefer OrbokSettings, fall back to catalog setting.
    let locale = Locale::parse(&settings.locale)
        .or_else(|| {
            SettingsRepository::new(&catalog)
                .get::<String>("ui.locale")
                .ok()
                .flatten()
                .and_then(|s| Locale::parse(&s))
        })
        .unwrap_or_default();

    // Verify embedding model files (design §startup-verify).
    let outcome = verify_embedding_model(settings.embedding_model_dir.as_deref());
    tracing::info!("{}", orbok_workers::verify_outcome_summary(&outcome));

    let (capability, wizard) = build_capability_and_wizard(outcome, &settings);

    let mut state = AppState::default();
    state.locale = locale;
    state.capability = capability;
    state.wizard = wizard;
    Ok(state)
}

/// Determine search capability and wizard state from the verify outcome.
fn build_capability_and_wizard(
    outcome: VerifyOutcome,
    _settings: &OrbokSettings,
) -> (SearchCapability, Option<WizardState>) {
    match outcome {
        VerifyOutcome::Ready => (SearchCapability::Hybrid, None),
        VerifyOutcome::NotConfigured => {
            (SearchCapability::KeywordOnly, Some(WizardState::NotConfigured))
        }
        VerifyOutcome::FilesInvalid { model_dir, issues } => {
            let checks: Vec<WizardFileCheck> = orbok_workers::model_verifier::REQUIRED_MODEL_FILES
                .iter()
                .map(|rel| {
                    let found = !issues.iter().any(|i| i.relative_path == *rel);
                    WizardFileCheck {
                        relative_path: rel.to_string(),
                        found,
                        size_mb: None,
                    }
                })
                .collect();
            let wizard = WizardState::FileMissing { previous_dir: model_dir };
            (SearchCapability::KeywordOnly, Some(wizard))
        }
    }
}

/// Execute a keyword/hybrid search and convert results to UI structs.
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

/// Headless backend validation (`--check` mode, RFC-017).
pub fn run_check() -> Result<(), Box<dyn std::error::Error>> {
    let dir = data_dir();
    tracing::info!(path = %dir.display(), "opening catalog");
    let catalog = open_catalog(&dir)?;
    let version = catalog.schema_version()?;
    let expected = orbok_db::migrations::latest_version();
    if version != expected {
        return Err(format!("schema version {version} != expected {expected}").into());
    }

    // Report model status in --check output.
    let settings = load_settings();
    let outcome = verify_embedding_model(settings.embedding_model_dir.as_deref());
    println!(
        "orbok --check OK  data_dir={}  schema_version={}  model={}",
        dir.display(),
        version,
        orbok_workers::verify_outcome_summary(&outcome)
    );
    Ok(())
}

/// Persist locale to the catalog (called when the user changes language).
pub fn persist_locale(catalog: &Catalog, locale: &Locale) -> OrbokResult<()> {
    SettingsRepository::new(catalog).set("ui.locale", &locale.as_str().to_string())
}

/// Persist the validated model directory to `OrbokSettings` (called when
/// the user completes the wizard and accepts a model folder).
pub fn persist_model_dir(model_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = load_settings();
    settings.embedding_model_dir = Some(model_dir.to_string());
    crate::settings::save_settings(&settings)
        .map_err(|e| format!("settings save failed: {e:?}"))?;
    Ok(())
}
