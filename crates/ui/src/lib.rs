//! # orbok-ui
//!
//! The orbok GUI layer: snora 0.25 (iced 0.14) views, the navigation
//! shell, and the typed i18n message catalog.
//!
//! Boundary rules (RFC-027):
//! - this crate performs **no file-system access** and **no database
//!   access** — `orbok-app` loads data through backend services and
//!   hands plain view-model structs to these views;
//! - every user-visible string goes through the [`i18n`] catalog
//!   (RFC-031): adding a [`i18n::Locale`] without translating every
//!   [`i18n::MessageKey`] is a compile error.

/// Lucide icon font bytes — register with iced before launching the app.
/// Re-exported from `snora::lucide`; no direct `lucide-icons` dep needed.
pub use snora::lucide::LUCIDE_FONT_BYTES;

pub mod components;
pub mod i18n;
pub mod notice;
pub mod shell;
pub mod state;
pub mod theme;
pub mod views;

#[cfg(test)]
mod tests;

pub use shell::OrbokApp;
pub use state::{
    AppState, IndexHealth, Message, NavGroup, SourceCard, ViewId, WizardFileCheck, WizardState,
};
pub use theme::Theme;
