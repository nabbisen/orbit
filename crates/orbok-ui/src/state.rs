//! Headless UI state (view models) and the message vocabulary.
//!
//! Everything here is plain data — testable without a display server.
//! `orbok-app` populates these structs from backend services; views
//! render them; `update` mutates them. No iced types appear in this
//! module so state logic stays UI-framework-agnostic.

use crate::i18n::Locale;
use orbok_models::SearchCapability;
use orbok_search::SearchMode;

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
    pub health: IndexHealth,
    pub sources: Vec<SourceCard>,
    pub capability: SearchCapability,
    pub storage_total_bytes: u64,
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
            health: IndexHealth::default(),
            sources: Vec::new(),
            capability: SearchCapability::KeywordOnly,
            storage_total_bytes: 0,
        }
    }
}

/// UI messages.
#[derive(Debug, Clone)]
pub enum Message {
    Switch(ViewId),
    QueryChanged(String),
    SubmitSearch,
    SearchResultsReady(Vec<SearchResultDisplay>),
    SearchError(String),
    SetSearchMode(SearchMode),
    SetLocale(Locale),
}

impl AppState {
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::Switch(view) => self.active_view = *view,
            Message::QueryChanged(query) => self.query = query.clone(),
            Message::SubmitSearch => {
                let trimmed = self.query.trim();
                if !trimmed.is_empty() {
                    self.last_query = Some(trimmed.to_string());
                    self.search_running = true;
                    self.search_results.clear();
                }
            }
            Message::SearchResultsReady(results) => {
                self.search_results = results.clone();
                self.search_running = false;
            }
            Message::SearchError(_) => {
                self.search_running = false;
            }
            Message::SetSearchMode(mode) => self.search_mode = *mode,
            Message::SetLocale(locale) => self.locale = *locale,
        }
    }
}
