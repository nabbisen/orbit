//! orbok application binary.
//!
//! Startup sequence (RFC-027 §architecture, NFR-022 lazy loading):
//! 1. resolve local data directory;
//! 2. open catalog and run migrations (aborts on failure, RFC-002 §6.2);
//! 3. load persisted locale from settings;
//! 4. `--check`: validate backend bootstrap headlessly and exit;
//! 5. otherwise: launch the GUI.

mod bootstrap;

use orbok_ui::OrbokApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--check") {
        return bootstrap::run_check();
    }

    let state = bootstrap::load_initial_state()?;
    iced::application(
        move || OrbokApp::with_state(state.clone()),
        OrbokApp::update,
        OrbokApp::view,
    )
    .title(|app: &OrbokApp| app.title())
    .run()?;
    Ok(())
}
