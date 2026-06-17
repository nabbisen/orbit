//! Typed i18n message catalog (RFC-031).
//!
//! Compile-time completeness: each locale module implements one
//! exhaustive `match` over [`MessageKey`]. Adding a key without adding
//! every translation fails the build — there is no runtime fallback
//! path to hide a missing string.
//!
//! Parameterized messages are plain functions (RFC-031 §5.3) so the
//! compiler also checks their arguments.

pub mod en;
pub mod ja;

use serde::{Deserialize, Serialize};

/// Supported UI locales. Default English; persisted in the catalog
/// under the `ui.locale` setting (read/written by `orbok-app`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    #[default]
    En,
    Ja,
}

impl Locale {
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Ja];

    /// Setting string stored in `app_settings` (`"en"` / `"ja"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ja => "ja",
        }
    }

    pub fn parse(s: &str) -> Option<Locale> {
        match s {
            "en" => Some(Locale::En),
            "ja" => Some(Locale::Ja),
            _ => None,
        }
    }

    /// Self-described language name, shown in the language picker.
    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Ja => "日本語",
        }
    }
}

/// Every fixed UI string. One variant per string; views never embed
/// literals (RFC-031 §6 rule 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKey {
    // Application chrome
    AppTitle,
    LocalOnlyBadge,
    // Navigation
    NavSearch,
    NavSources,
    NavIndexing,
    NavStorage,
    NavModels,
    NavAi,
    NavSettings,
    // Search view
    SearchPlaceholder,
    SearchButton,
    SearchNoSourcesTitle,
    SearchNoSourcesBody,
    SearchAddSource,
    SearchNoResults,
    SearchKeywordOnlyNotice,
    // Sources view
    SourcesTitle,
    SourcesEmptyTitle,
    SourcesEmptyBody,
    SourcesAddFolder,
    SourcesStatusActive,
    SourcesStatusPaused,
    SourcesStatusMissing,
    // Indexing view
    IndexingTitle,
    IndexingIdle,
    IndexingHealthIndexed,
    IndexingHealthStale,
    IndexingHealthFailed,
    IndexingHealthQueued,
    // Storage view
    StorageTitle,
    StorageIntro,
    StorageGroupSearchIndex,
    StorageGroupModels,
    StorageGroupCaches,
    StorageSafeCleanupHeading,
    StorageClearSnippets,
    StorageClearSearchCache,
    StorageDangerHeading,
    StorageResetCatalog,
    StorageResetWarning,
    // Models view
    ModelsTitle,
    ModelsEmbeddingRole,
    ModelsRerankerRole,
    ModelsStatusAvailable,
    ModelsStatusMissing,
    ModelsKeywordOnlyHint,
    // Settings view
    SettingsTitle,
    SettingsLanguageHeading,
    SettingsPrivacyHeading,
    SettingsAdvancedHeading,
    SettingsAdvancedOn,
    SettingsAdvancedOff,
    SettingsAdvancedHint,
    SettingsPrivacyLocalOnly,
    // Search modes (RFC-009 §8)
    SearchModeLabel,
    SearchModeAuto,
    SearchModeExact,
    SearchModeConceptual,
    SearchModeFast,
    // Match badges
    BadgeKeyword,
    BadgeSemantic,
    BadgeFused,
    // Startup wizard (design §wizard)
    WizardTitleNotConfigured,
    WizardTitleFileMissing,
    WizardTitleValidating,
    WizardTitleReady,
    WizardBodyNotConfigured,
    WizardBodyFileMissing,
    WizardFilesNeededLabel,
    WizardDownloadHint,
    WizardPathInputPlaceholder,
    WizardActionLocate,
    WizardActionValidate,
    WizardActionUseModel,
    WizardActionContinue,
    WizardPathPlaceholder,
    WizardDownloadAction,
    WizardDownloadProgress,
    WizardActionSkip,
    WizardPreviousPathLabel,
    WizardValidationOk,
    WizardValidationFail,
    WizardReadyBody,
    // Common actions
    Cancel,
    Confirm,
}

/// Translate a fixed message. The per-locale functions are exhaustive
/// matches — completeness is enforced by the compiler.
pub fn tr(locale: Locale, key: MessageKey) -> &'static str {
    match locale {
        Locale::En => en::message(key),
        Locale::Ja => ja::message(key),
    }
}

/// Parameterized: "812 files indexed".
pub fn files_indexed(locale: Locale, count: u64) -> String {
    match locale {
        Locale::En => format!("{count} files indexed"),
        Locale::Ja => format!("{count} 件のファイルをインデックス済み"),
    }
}

/// Parameterized: source card summary line.
pub fn source_summary(locale: Locale, indexed: u64, stale: u64, failed: u64) -> String {
    match locale {
        Locale::En => format!("{indexed} indexed · {stale} stale · {failed} failed"),
        Locale::Ja => format!("インデックス済み {indexed} · 要更新 {stale} · 失敗 {failed}"),
    }
}

/// Parameterized: "3 results".
pub fn search_result_count(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => format!("{count} result{}", if count == 1 { "" } else { "s" }),
        Locale::Ja => format!("{count} 件の結果"),
    }
}
