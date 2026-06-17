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
mod download;
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
        move |app: &mut OrbokApp, message: Message| -> iced::Task<Message> {
            // Handle backend effects before passing message to UI state.
            match &message {
                Message::DownloadModel => {
                    let dest = data_dir
                        .join("models")
                        .join("multilingual-e5-small");
                    std::fs::create_dir_all(&dest).ok();
                    let dest_str = dest.to_string_lossy().to_string();
                    app.update(Message::DownloadStarted { dest_dir: dest_str });
                    let (tx, rx) = iced::futures::channel::mpsc::channel::<Message>(64);
                    tokio::spawn(download::run(dest, tx));
                    return iced::Task::stream(rx);
                }
                Message::WizardValidate => {
                    let path = app.state.wizard_path_input.trim().to_string();
                    let outcome = verify_embedding_model(Some(&path));
                    let (checks, all_ok) = build_wizard_checks(&outcome, &path);
                    app.update(Message::WizardChecked {
                        model_dir: path,
                        checks,
                        all_ok,
                    });
                    return iced::Task::none();
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
                    // Open the OS-native folder picker.
                    // `pick_folder()` is synchronous; it blocks the update loop
                    // while the dialog is open, which is expected for a modal dialog.
                    let picked = rfd::FileDialog::new()
                        .set_title("Select folder to search")
                        .pick_folder();
                    if let Some(folder) = picked {
                        let path = folder.to_string_lossy().to_string();
                        app.update(Message::SourcePathChanged(path.clone()));
                        if let Ok(catalog) = orbok_db::Catalog::open(&catalog_path) {
                            let cache = orbok_cache::CacheService::new(&data_dir);
                            match bootstrap::add_source(&catalog, &path) {
                                Ok(card) => {
                                    let source_id = card.source_id.clone();
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
                    return iced::Task::none();
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
                                    return iced::Task::none();
                                }
                                Err(e) => {
                                    app.update(message.clone());
                                    app.update(Message::SearchError(e.to_string()));
                                    return iced::Task::none();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            app.update(message);
            iced::Task::none()
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
