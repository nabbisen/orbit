//! Headless UI state (view models) and the message vocabulary.
//!
//! Everything here is plain data — testable without a display server.
//! `orbok-app` populates these structs from backend services; views
//! render them; `update` mutates them. No iced types appear in this
//! module so state logic stays UI-framework-agnostic.

use crate::i18n::Locale;
use crate::notice::UserNotice;
use orbok_models::SearchCapability;
use orbok_search::SearchMode;

/// Top-level navigation group for the two-level sidebar + tab layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavGroup {
    Search,
    Ai,
    Settings,
}

/// Top-level pages (GUI external design §3.1 order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewId {
    Search,
    Sources,
    Indexing,
    Storage,
    Models,
    Settings,
}

impl ViewId {
    pub const ALL: &'static [ViewId] = &[
        ViewId::Search,
        ViewId::Sources,
        ViewId::Indexing,
        ViewId::Storage,
        ViewId::Models,
        ViewId::Settings,
    ];

    /// Which top-level navigation group this view belongs to.
    pub fn group(self) -> NavGroup {
        match self {
            ViewId::Search | ViewId::Sources => NavGroup::Search,
            ViewId::Indexing | ViewId::Storage | ViewId::Models => NavGroup::Ai,
            ViewId::Settings => NavGroup::Settings,
        }
    }

    /// Default view to activate when the user first enters a group.
    pub fn group_default(group: NavGroup) -> Self {
        match group {
            NavGroup::Search => ViewId::Search,
            NavGroup::Ai => ViewId::Indexing,
            NavGroup::Settings => ViewId::Settings,
        }
    }
}

/// Sidebar index-health summary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IndexHealth {
    pub indexed: u64,
    pub stale: u64,
    pub failed: u64,
    pub queued: u64,
}

/// One source card for the Sources view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCard {
    pub display_name: String,
    pub display_path: String,
    pub indexed: u64,
    pub stale: u64,
    pub failed: u64,
    pub active: bool,
    pub source_id: String,
}

/// A search result ready for display — pure data, no backend types
/// (RFC-027 boundary rule).
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultDisplay {
    pub display_path: String,
    pub title: Option<String>,
    pub heading_path: Option<String>,
    pub snippet: Option<String>,
    pub keyword_rank: u32,
    pub badges: Vec<String>,
}

/// One required file and its check result shown in the wizard.
#[derive(Debug, Clone, PartialEq)]
pub struct WizardFileCheck {
    pub relative_path: String,
    pub found: bool,
    pub size_mb: Option<f64>,
}

/// Which stage of the startup wizard the user is on.
#[derive(Debug, Clone, PartialEq)]
pub enum WizardState {
    /// First launch or model never configured.
    NotConfigured,
    /// Was configured, but files are gone.
    FileMissing {
        previous_dir: String,
        checks: Vec<WizardFileCheck>,
    },
    /// User submitted a path; file checks complete.
    Checked {
        model_dir: String,
        checks: Vec<WizardFileCheck>,
        all_ok: bool,
    },
    /// All files verified — ready to proceed.
    Ready { model_dir: String },
    /// HuggingFace download in progress.
    Downloading {
        dest_dir: String,
        /// Filename currently being downloaded.
        current_file: String,
        bytes: u64,
        total: Option<u64>,
        files_done: u32,
        files_total: u32,
    },
}

/// The whole-app view model.
#[derive(Debug, Clone)]
pub struct AppState {
    pub active_view: ViewId,
    pub locale: Locale,
    pub query: String,
    pub last_query: Option<String>,
    pub search_mode: SearchMode,
    pub search_results: Vec<SearchResultDisplay>,
    pub search_running: bool,
    pub selected_result: Option<usize>,
    pub storage_rows: Vec<(String, u64, u64)>,
    pub health: IndexHealth,
    pub sources: Vec<SourceCard>,
    pub capability: SearchCapability,
    pub storage_total_bytes: u64,
    /// Active startup wizard, or `None` when startup succeeded.
    pub wizard: Option<WizardState>,
    /// Text-input path the user is typing in the wizard.
    pub wizard_path_input: String,
    /// Text input for the "add source" path field.
    pub source_path_input: String,
    /// When false (default), hide technical detail. Mature users can toggle on.
    pub show_advanced: bool,
    /// Active user-facing notice (problem or confirmation), or `None`.
    pub notice: Option<UserNotice>,
    /// Awaiting user confirmation before running reset catalog.
    pub confirm_reset: bool,
    /// Snora Design tokens, selected by `high_contrast`. Drives notice colors
    /// and (incrementally) other design-system surfaces.
    pub tokens: snora::design::Tokens,
    /// When true, use the high-contrast token preset (accessibility).
    pub high_contrast: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_view: ViewId::Search,
            locale: Locale::default(),
            query: String::new(),
            last_query: None,
            search_mode: SearchMode::Auto,
            search_results: Vec::new(),
            search_running: false,
            selected_result: None,
            storage_rows: Vec::new(),
            health: IndexHealth::default(),
            sources: Vec::new(),
            capability: SearchCapability::KeywordOnly,
            storage_total_bytes: 0,
            wizard: None,
            wizard_path_input: String::new(),
            source_path_input: String::new(),
            show_advanced: false,
            notice: None,
            confirm_reset: false,
            tokens: snora::design::Tokens::light(),
            high_contrast: false,
        }
    }
}

/// UI messages.
#[derive(Debug, Clone)]
pub enum Message {
    Switch(ViewId),
    SwitchGroup(NavGroup),
    ToggleAdvanced,
    ToggleHighContrast,
    ShowNotice(UserNotice),
    ClearNotice,
    // Storage cleanup
    CleanSnippets,
    CleanSearchCache,
    AskResetCatalog,
    ConfirmResetCatalog,
    CancelResetCatalog,
    CleanupDone, // backend notifies completion
    // Wizard navigation
    WizardBack,
    QueryChanged(String),
    SubmitSearch,
    SearchResultsReady(Vec<SearchResultDisplay>),
    SearchError(String),
    SelectResult(usize),
    OpenSourceFile(String),
    SetSearchMode(SearchMode),
    PersistLocale(Locale),
    SetLocale(Locale),
    StorageDataReady(Vec<(String, u64, u64)>),
    // Startup wizard
    WizardPathChanged(String),
    WizardValidate,
    WizardChecked {
        model_dir: String,
        checks: Vec<WizardFileCheck>,
        all_ok: bool,
    },
    WizardAccept,
    WizardSkip,
    // Source management
    SourcePathChanged(String),
    RequestAddSource,
    SourceAdded(SourceCard),
    SourceRemoved(String), // source_id
    ScanCompleted(IndexHealth),
    // Download
    DownloadModel,
    DownloadStarted {
        dest_dir: String,
    },
    DownloadFileProgress {
        file: String,
        bytes: u64,
        total: Option<u64>,
        files_done: u32,
        files_total: u32,
    },
    DownloadAllComplete {
        dest_dir: String,
    },
    DownloadFailed(String),
    // Startup population
    HealthUpdated(IndexHealth),
    SourcesLoaded(Vec<SourceCard>),
}

impl AppState {
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::Switch(view) => self.active_view = *view,
            Message::SwitchGroup(group) => self.active_view = ViewId::group_default(*group),
            Message::ToggleAdvanced => self.show_advanced = !self.show_advanced,
            Message::ToggleHighContrast => {
                self.high_contrast = !self.high_contrast;
                self.tokens = if self.high_contrast {
                    snora::design::Tokens::high_contrast_light()
                } else {
                    snora::design::Tokens::light()
                };
            }
            Message::AskResetCatalog => self.confirm_reset = true,
            Message::CancelResetCatalog => self.confirm_reset = false,
            Message::ConfirmResetCatalog => {
                self.confirm_reset = false;
                // Actual reset handled in orbok-app; UI pre-clears state.
                self.sources.clear();
                self.health = crate::state::IndexHealth::default();
                self.search_results.clear();
                self.storage_rows.clear();
                self.storage_total_bytes = 0;
            }
            Message::CleanSnippets | Message::CleanSearchCache => {
                // Actual work done in orbok-app; state update arrives via CleanupDone.
            }
            Message::CleanupDone => {
                self.notice = Some(UserNotice::PreviewsCleared);
            }
            Message::WizardBack => {
                // Return to the initial setup step.
                self.wizard = Some(crate::state::WizardState::NotConfigured);
                self.wizard_path_input = String::new();
            }
            Message::ShowNotice(n) => self.notice = Some(n.clone()),
            Message::ClearNotice => self.notice = None,
            Message::QueryChanged(query) => self.query = query.clone(),
            Message::SubmitSearch => {
                let trimmed = self.query.trim();
                if !trimmed.is_empty() {
                    self.last_query = Some(trimmed.to_string());
                    self.search_running = true;
                    self.search_results.clear();
                    self.selected_result = None;
                }
            }
            Message::SearchResultsReady(results) => {
                self.search_results = results.clone();
                self.search_running = false;
                self.selected_result = None;
                self.notice = None;
            }
            Message::SearchError(_) => {
                self.search_running = false;
                self.notice = Some(UserNotice::SearchDidNotFinish);
            }
            Message::SelectResult(idx) => self.selected_result = Some(*idx),
            Message::OpenSourceFile(_) => {} // handled by orbok-app
            Message::SetSearchMode(mode) => self.search_mode = *mode,
            Message::PersistLocale(locale) | Message::SetLocale(locale) => self.locale = *locale,
            Message::StorageDataReady(rows) => self.storage_rows = rows.clone(),
            Message::WizardPathChanged(p) => self.wizard_path_input = p.clone(),
            Message::WizardValidate => {} // handled in orbok-app update
            Message::WizardChecked {
                model_dir,
                checks,
                all_ok,
            } => {
                self.wizard = Some(if *all_ok {
                    WizardState::Ready {
                        model_dir: model_dir.clone(),
                    }
                } else {
                    WizardState::Checked {
                        model_dir: model_dir.clone(),
                        checks: checks.clone(),
                        all_ok: false,
                    }
                });
            }
            Message::WizardAccept => {
                // orbok-app writes the model dir to OrbokSettings; ui
                // transitions to full capability.
                self.capability = SearchCapability::Hybrid;
                self.wizard = None;
                self.wizard_path_input = String::new();
            }
            Message::WizardSkip => {
                self.capability = SearchCapability::KeywordOnly;
                self.wizard = None;
                self.wizard_path_input = String::new();
            }
            Message::DownloadModel => {
                // Transition handled in orbok-app main.rs (needs the data_dir).
                // The UI just switches to a "waiting" state until DownloadStarted arrives.
            }
            Message::DownloadStarted { dest_dir } => {
                self.wizard = Some(WizardState::Downloading {
                    dest_dir: dest_dir.clone(),
                    current_file: String::new(),
                    bytes: 0,
                    total: None,
                    files_done: 0,
                    files_total: 2,
                });
            }
            Message::DownloadFileProgress {
                file,
                bytes,
                total,
                files_done,
                files_total,
            } => {
                if let Some(WizardState::Downloading {
                    current_file,
                    bytes: b,
                    total: t,
                    files_done: fd,
                    files_total: ft,
                    ..
                }) = &mut self.wizard
                {
                    *current_file = file.clone();
                    *b = *bytes;
                    *t = *total;
                    *fd = *files_done;
                    *ft = *files_total;
                }
            }
            Message::DownloadAllComplete { dest_dir } => {
                // Switch directly to wizard-accepted flow.
                self.wizard = Some(WizardState::Ready {
                    model_dir: dest_dir.clone(),
                });
            }
            Message::DownloadFailed(_reason) => {
                // Return to NotConfigured so the user can try again.
                self.wizard = Some(WizardState::NotConfigured);
            }
            Message::SourcePathChanged(p) => self.source_path_input = p.clone(),
            Message::RequestAddSource => {} // handled in orbok-app
            Message::SourceAdded(card) => {
                self.sources.push(card.clone());
                self.source_path_input = String::new();
                self.notice = Some(UserNotice::FolderAdded);
            }
            Message::SourceRemoved(id) => self.sources.retain(|s| s.source_id != *id),
            Message::ScanCompleted(health) | Message::HealthUpdated(health) => {
                self.health = *health;
                // Update per-source counts from the fresh health data.
            }
            Message::SourcesLoaded(cards) => self.sources = cards.clone(),
        }
    }
}
