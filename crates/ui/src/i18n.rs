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

    /// Detect the preferred locale from the operating system environment.
    /// Checks `LANG` and `LANGUAGE` in that order. Returns `None` if
    /// neither variable is set or contains a recognised language code.
    /// Japanese is recognised when the value starts with `ja` (e.g. `ja`,
    /// `ja_JP`, `ja_JP.UTF-8`).
    pub fn from_env() -> Option<Locale> {
        for var in &["LANG", "LANGUAGE"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_lowercase();
                if lower.starts_with("ja") {
                    return Some(Locale::Ja);
                }
                if lower.starts_with("en") {
                    return Some(Locale::En);
                }
            }
        }
        None
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
    NoticeDownloadFailTitle,
    NoticeDownloadFailBody,
    NoticeFolderFailTitle,
    NoticeFolderFailBody,
    NoticeSearchFailTitle,
    NoticeSearchFailBody,
    NoticeFilesMissingTitle,
    NoticeFilesMissingBody,
    NoticeFolderAddedTitle,
    NoticeFolderAddedBody,
    NoticeSearchReadyTitle,
    NoticeSearchReadyBody,
    NoticePreviewsClearedTitle,
    NoticePreviewsClearedBody,
    NoticeActionTryAgain,
    NoticeActionChooseFolder,
    SettingsThemeHeading,
    ThemeSystem,
    ThemeLight,
    ThemeDark,
    ThemeHighContrastLight,
    ThemeHighContrastDark,
    // RFC-035 inclusive design
    SettingsTextScaleHeading,
    TextScaleDefault,
    TextScaleLarge,
    TextScaleLarger,
    SettingsReduceMotion,
    SettingsReduceMotionHint,
    SettingsCvdNote,
    NoticeSensitiveSourceTitle,
    NoticeSensitiveSourceBody,
    NoticeDismiss,
    Cancel,
    Confirm,
    // RFC-041: Search, Narrow Results, Browse Around
    SearchNarrowResults,
    SearchNarrowedBy,
    SearchMoreWays,
    SearchClearFilters,
    SearchNoResultsFiltered,
    SearchNoResultsFilteredBody,
    SearchInThisFolder,
    SearchShowNearby,
    SearchShowSimilar,
    SearchResultsUpdating,
    SearchPreparingFolder,
    SearchPartialReadiness,
    // RFC-041 filter labels
    FilterKind,
    FilterChanged,
    FilterSearchIn,
    FilterReadyStatus,
    FilterKindPdfs,
    FilterKindNotes,
    FilterKindCode,
    FilterKindDocuments,
    FilterKindSpreadsheets,
    FilterChangedToday,
    FilterChangedThisWeek,
    FilterChangedThisMonth,
    FilterChangedAnyTime,
    FilterAllFolders,
    // RFC-037: Source lifecycle
    SourceStateReady,
    SourceStatePreparing,
    SourceStateNeedsUpdate,
    SourceStatePaused,
    SourceStateFolderNotFound,
    SourceStateCannotOpen,
    SourceStateRemoved,
    SourceActionCheckAgain,
    SourceActionPrepareAgain,
    SourceActionChooseFolderAgain,
    SourceActionRemoveFromOrbok,
    SourceFolderNotFoundDetail,
    SourceFilesNotDeletedNotice,
    SourceManyFilesChanged,
    SourcePausePreparation,
    SourceResumePreparation,
    // RFC-038: Result trust badges and recovery
    TrustNeedsUpdate,
    TrustFileNotFound,
    TrustStillBeingPrepared,
    TrustPartlyPrepared,
    TrustCannotOpen,
    TrustActionPrepareAgain,
    TrustActionCheckFolder,
    TrustActionRemoveFromResults,
    TrustActionOpenAnyway,
    TrustActionShowInFolder,
    TrustActionViewDetails,
    TrustFileChangedDetail,
    TrustFileNotFoundDetail,
    TrustPartlyPreparedDetail,
    TrustScannedPdfDetail,
    TrustSomePagesDetail,
    TrustSizeLimitDetail,
    TrustCannotOpenDetail,
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

/// Locale-aware byte/storage size formatting (RFC-035 §5.5).
/// Routes views away from ad-hoc `format!("{gib:.3} GiB total")` calls.
pub fn fmt_gib(locale: Locale, gib: f64) -> String {
    match locale {
        Locale::En => format!("{gib:.3} GiB total"),
        Locale::Ja => format!("合計 {gib:.3} GiB"),
    }
}

/// Locale-aware MiB bucket formatting for the storage view friendly buckets.
pub fn fmt_mib_bucket(locale: Locale, label: &str, mib: f64) -> String {
    match locale {
        Locale::En => format!("  {label}: {mib:.1} MiB"),
        Locale::Ja => format!("  {label}: {mib:.1} MiB"),
    }
}

/// Locale-aware advanced storage row formatting (category + count).
pub fn fmt_storage_row(locale: Locale, category: &str, mib: f64, count: u64) -> String {
    match locale {
        Locale::En => format!("  {category}: {mib:.1} MiB ({count} items)"),
        Locale::Ja => format!("  {category}: {mib:.1} MiB（{count} 件）"),
    }
}

/// Locale-aware last-query display (search view "no results" state).
pub fn fmt_query(locale: Locale, query: &str) -> String {
    match locale {
        Locale::En => format!("Query: {query}"),
        Locale::Ja => format!("検索語: {query}"),
    }
}
