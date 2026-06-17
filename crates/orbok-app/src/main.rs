//! orbok application binary.
//!
//! Startup sequence (RFC-027, design §startup):
//! 1. parse flags (--version, --portable, --check)
//! 2. resolve data directory
//! 3. open catalog, run migrations, run startup recovery (RFC-018)
//! 4. load OrbokSettings, verify model files → build AppState
//! 5. if wizard active: show wizard until resolved or skipped
//! 6. launch main GUI

mod bootstrap;
mod settings;

use orbok_ui::{Message, OrbokApp};
use orbok_ui::state::WizardFileCheck;
use orbok_workers::{VerifyOutcome, verify_embedding_model};
use orbok_workers::model_verifier::REQUIRED_MODEL_FILES;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("orbok {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let portable = args.iter().any(|a| a == "--portable");
    if portable {
        eprintln!("orbok: portable mode — data directory: ./orbok-data/");
    }
    if args.iter().any(|a| a == "--check") {
        return bootstrap::run_check();
    }

    let state = bootstrap::load_initial_state()?;
    let data_dir = bootstrap::data_dir_for_args(portable);
    let catalog_path = data_dir.join(orbok_db::CATALOG_FILE_NAME);

    iced::application(
        move || OrbokApp::with_state(state.clone()),
        move |app: &mut OrbokApp, message: Message| {
            // Handle backend effects before passing message to UI state.
            match &message {
                Message::WizardValidate => {
                    let path = app.state.wizard_path_input.trim().to_string();
                    let outcome = verify_embedding_model(Some(&path));
                    let (checks, all_ok) = build_wizard_checks(&outcome, &path);
                    app.update(Message::WizardChecked {
                        model_dir: path,
                        checks,
                        all_ok,
                    });
                    return;
                }
                Message::WizardAccept => {
                    // Persist the accepted model directory to OrbokSettings.
                    if let Some(orbok_ui::state::WizardState::Ready { model_dir }) =
                        &app.state.wizard
                    {
                        if let Err(e) = bootstrap::persist_model_dir(model_dir.as_str()) {
                            tracing::error!("failed to save model dir: {e}");
                        }
                    }
                }
                Message::RequestAddSource => {
                    let path = app.state.source_path_input.trim().to_string();
                    if !path.is_empty() {
                        if let Ok(catalog) = orbok_db::Catalog::open(&catalog_path) {
                            let cache = orbok_cache::CacheService::new(&data_dir);
                            match bootstrap::add_source(&catalog, &path) {
                                Ok(card) => {
                                    let source_id = card.source_id.clone();
                                    app.update(message.clone());
                                    app.update(Message::SourceAdded(card));
                                    match bootstrap::scan_and_index_source(&catalog, &cache, &source_id) {
                                        Ok(health) => app.update(Message::ScanCompleted(health)),
                                        Err(e) => tracing::error!("scan failed: {e}"),
                                    }
                                }
                                Err(e) => tracing::error!("add source failed: {e}"),
                            }
                        }
                    }
                    return;
                }
                Message::SourceRemoved(source_id) => {
                    if let Ok(catalog) = orbok_db::Catalog::open(&catalog_path) {
                        let _ = bootstrap::remove_source(&catalog, source_id);
                    }
                }
                Message::PersistLocale(locale) => {
                    if let Ok(catalog) = orbok_db::Catalog::open(&catalog_path) {
                        let _ = bootstrap::persist_locale(&catalog, locale);
                    }
                }
                Message::SubmitSearch => {
                    let query = app.state.query.trim().to_string();
                    if !query.is_empty() {
                        if let Ok(catalog) = orbok_db::Catalog::open(&catalog_path) {
                            match bootstrap::run_search(&catalog, &query, 20) {
                                Ok(results) => {
                                    app.update(message.clone());
                                    app.update(Message::SearchResultsReady(results));
                                    return;
                                }
                                Err(e) => {
                                    app.update(message.clone());
                                    app.update(Message::SearchError(e.to_string()));
                                    return;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            app.update(message);
        },
        OrbokApp::view,
    )
    .title(|app: &OrbokApp| app.title())
    .font(orbok_ui::LUCIDE_FONT_BYTES)
    .run()?;
    Ok(())
}

/// Convert a `VerifyOutcome` into the file check list shown in the wizard.
fn build_wizard_checks(
    outcome: &VerifyOutcome,
    _path: &str,
) -> (Vec<WizardFileCheck>, bool) {
    match outcome {
        VerifyOutcome::Ready => {
            let checks = REQUIRED_MODEL_FILES
                .iter()
                .map(|rel| WizardFileCheck {
                    relative_path: rel.to_string(),
                    found: true,
                    size_mb: None,
                })
                .collect();
            (checks, true)
        }
        VerifyOutcome::FilesInvalid { issues, .. } => {
            let checks = REQUIRED_MODEL_FILES
                .iter()
                .map(|rel| WizardFileCheck {
                    relative_path: rel.to_string(),
                    found: !issues.iter().any(|i| i.relative_path == *rel),
                    size_mb: None,
                })
                .collect();
            (checks, false)
        }
        VerifyOutcome::NotConfigured => {
            let checks = REQUIRED_MODEL_FILES
                .iter()
                .map(|rel| WizardFileCheck {
                    relative_path: rel.to_string(),
                    found: false,
                    size_mb: None,
                })
                .collect();
            (checks, false)
        }
    }
}
