//! Theme selection and token-reading helpers (RFC-032).
//!
//! This module is the single place the UI reads design *values*. View code
//! never uses literal font sizes, paddings, or colors; it calls these helpers,
//! which read the active [`snora::design::Tokens`] bundle held in
//! [`crate::state::AppState::tokens`].
//!
//! [`Theme`] mirrors the locale model (RFC-031): a small typed enum, an
//! exhaustive `match` to its concrete token preset, a setting string for
//! persistence, and an OS-environment resolver for the `System` value. snora
//! remains the sole gateway to the design vocabulary — these helpers wrap the
//! `snora::design` style bridge and never invent values.

use iced::Pixels;
use serde::{Deserialize, Serialize};
use snora::design::Tokens;

use crate::i18n::MessageKey;

/// User-selectable UI theme. `System` is resolved to a concrete variant at
/// startup (in `orbok-app`) via [`Theme::from_env`); the other four map
/// directly to the built-in Snora Design presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Follow the operating system preference (resolved at startup).
    #[default]
    System,
    /// Calm light theme.
    Light,
    /// Low-glare dark theme.
    Dark,
    /// High-contrast light theme (accessibility).
    HighContrastLight,
    /// High-contrast dark theme (accessibility).
    HighContrastDark,
}

impl Theme {
    /// All selectable themes, in display order (System first).
    pub const ALL: &'static [Theme] = &[
        Theme::System,
        Theme::Light,
        Theme::Dark,
        Theme::HighContrastLight,
        Theme::HighContrastDark,
    ];

    /// Setting string stored in `OrbokSettings` (`settings.json`).
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::HighContrastLight => "high_contrast_light",
            Theme::HighContrastDark => "high_contrast_dark",
        }
    }

    /// Parse a stored setting string back into a [`Theme`].
    pub fn parse(s: &str) -> Option<Theme> {
        Some(match s {
            "system" => Theme::System,
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "high_contrast_light" => Theme::HighContrastLight,
            "high_contrast_dark" => Theme::HighContrastDark,
            _ => return None,
        })
    }

    /// The i18n key for this theme's display name (used by the Settings picker).
    pub fn label_key(self) -> MessageKey {
        match self {
            Theme::System => MessageKey::ThemeSystem,
            Theme::Light => MessageKey::ThemeLight,
            Theme::Dark => MessageKey::ThemeDark,
            Theme::HighContrastLight => MessageKey::ThemeHighContrastLight,
            Theme::HighContrastDark => MessageKey::ThemeHighContrastDark,
        }
    }

    /// The concrete Snora Design token bundle for this theme.
    ///
    /// `System` falls back to the light preset here; the real OS resolution
    /// happens once at startup in `orbok-app` via [`Theme::from_env`], which
    /// substitutes a concrete variant before token construction. Selecting
    /// `System` at runtime therefore applies the light bundle until the next
    /// launch (RFC-032 §5.5 — `System` resolved once at startup).
    pub fn tokens(self) -> Tokens {
        match self {
            Theme::Light | Theme::System => Tokens::light(),
            Theme::Dark => Tokens::dark(),
            Theme::HighContrastLight => Tokens::high_contrast_light(),
            Theme::HighContrastDark => Tokens::high_contrast_dark(),
        }
    }

    /// Best-effort resolution of the OS colour-scheme preference for the
    /// `System` theme. v1 honours an explicit `ORBOK_THEME` override (a
    /// concrete theme name) and otherwise returns `None`, leaving the caller
    /// to fall back to [`Theme::Light`]. A richer per-platform probe (desktop
    /// portal / Windows registry / `AppleInterfaceStyle`) is a tracked
    /// follow-up (RFC-032 §9). Mirrors [`crate::i18n::Locale::from_env`].
    pub fn from_env() -> Option<Theme> {
        let raw = std::env::var("ORBOK_THEME").ok()?;
        match Theme::parse(raw.trim()) {
            // An explicit `system` override is not itself a concrete answer.
            Some(Theme::System) | None => None,
            concrete => concrete,
        }
    }
}

// ── Typography helpers (wrap the snora style bridge) ──────────────────────
// Each returns an `iced::Pixels` derived from the active tokens' text roles,
// so views call e.g. `text(..).size(theme::body(&state.tokens))`.

/// Page / section heading size.
pub fn heading(t: &Tokens) -> Pixels {
    snora::design::style::text::heading_size(t)
}

/// Card / dialog / metric title size.
pub fn title(t: &Tokens) -> Pixels {
    snora::design::style::text::title_size(t)
}

/// Ordinary body text size.
pub fn body(t: &Tokens) -> Pixels {
    snora::design::style::text::body_size(t)
}

/// Secondary metadata / compact help size.
pub fn meta(t: &Tokens) -> Pixels {
    snora::design::style::text::body_small_size(t)
}

/// Button / chip / control label size.
pub fn label(t: &Tokens) -> Pixels {
    snora::design::style::text::label_size(t)
}
