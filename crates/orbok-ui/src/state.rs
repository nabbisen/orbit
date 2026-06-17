//! Headless UI state (view models) and the message vocabulary.
//!
//! Everything here is plain data — testable without a display server.
//! `orbok-app` populates these structs from backend services; views
//! render them; `update` mutates them. No iced types appear in this
//! module so state logic stays UI-framework-agnostic.

use crate::i18n::Locale;
use orbok_models::SearchCapability;

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
    /// Sidebar order (Search first — search-first navigation, GUI
    /// design §2.2).
    pub const ALL: &'static [ViewId] = &[
        ViewId::Search,
        ViewId::Sources,
        ViewId::Indexing,
        ViewId::Storage,
        ViewId::Models,
        ViewId::Settings,
    ];
}

/// Sidebar index-health summary (GUI design §5.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IndexHealth {
    pub indexed: u64,
    pub stale: u64,
    pub failed: u64,
    pub queued: u64,
}

/// One source card (GUI design §8.1), pre-localized display fields
/// excepted — status text is resolved at render time via i18n.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCard {
    pub display_name: String,
    pub display_path: String,
    pub indexed: u64,
    pub stale: u64,
    pub failed: u64,
    pub active: bool,
}

/// The whole-app view model.
#[derive(Debug, Clone)]
pub struct AppState {
    pub active_view: ViewId,
    pub locale: Locale,
    pub query: String,
    pub last_query: Option<String>,
    pub health: IndexHealth,
    pub sources: Vec<SourceCard>,
    pub capability: SearchCapability,
    /// Total orbok storage in bytes (Storage view headline).
    pub storage_total_bytes: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_view: ViewId::Search,
            locale: Locale::default(),
            query: String::new(),
            last_query: None,
            health: IndexHealth::default(),
            sources: Vec::new(),
            capability: SearchCapability::KeywordOnly,
            storage_total_bytes: 0,
        }
    }
}

/// UI messages. Backend-effecting intents (scan, cleanup, search) are
/// surfaced as messages here and executed by `orbok-app`'s update glue,
/// keeping this crate free of side effects.
#[derive(Debug, Clone)]
pub enum Message {
    Switch(ViewId),
    QueryChanged(String),
    SubmitSearch,
    SetLocale(Locale),
}

impl AppState {
    /// Pure state transition. Side-effect intents (e.g. running the
    /// search) are handled by the embedding application after calling
    /// this.
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::Switch(view) => self.active_view = *view,
            Message::QueryChanged(query) => self.query = query.clone(),
            Message::SubmitSearch => {
                let trimmed = self.query.trim();
                if !trimmed.is_empty() {
                    self.last_query = Some(trimmed.to_string());
                }
            }
            Message::SetLocale(locale) => self.locale = *locale,
        }
    }
}
