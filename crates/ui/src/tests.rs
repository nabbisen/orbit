//! orbok-ui test suite.
//!
//! This file is the module router. Tests live in submodules under `tests/`:
//!
//! | Module | Coverage |
//! |---|---|
//! | `i18n` | i18n catalog completeness, locale detection, parameterized messages |
//! | `state` | AppState transitions, theme/scale/motion, navigation, notices |
//! | `components` | RFC-033 adapter smoke tests and tone-mapping |
//! | `a11y` | RFC-034 contrast guard, keyboard map, RFC-035 CVD + scale |
//! | `smoke_views` | headless view-render smoke tests |
//!
//! `ALL_KEYS` is declared here because both `i18n` and tooling that
//! enumerates keys need a single canonical list.

pub mod a11y;
pub mod components;
pub mod i18n;
pub mod smoke_views;
pub mod state;

use crate::i18n::MessageKey;

/// Every MessageKey that must resolve to a non-empty string in every locale.
/// Update this list when adding or removing keys from the MessageKey enum.
pub const ALL_KEYS: &[MessageKey] = &[
    MessageKey::AppTitle,
    MessageKey::LocalOnlyBadge,
    MessageKey::NavSearch,
    MessageKey::NavSources,
    MessageKey::NavIndexing,
    MessageKey::NavStorage,
    MessageKey::NavModels,
    MessageKey::NavSettings,
    MessageKey::SearchPlaceholder,
    MessageKey::SearchButton,
    MessageKey::SearchNoSourcesTitle,
    MessageKey::SearchNoSourcesBody,
    MessageKey::SearchAddSource,
    MessageKey::SearchNoResults,
    MessageKey::SearchKeywordOnlyNotice,
    MessageKey::SearchModeLabel,
    MessageKey::SearchModeAuto,
    MessageKey::SearchModeExact,
    MessageKey::SearchModeConceptual,
    MessageKey::SourcesTitle,
    MessageKey::SourcesEmptyTitle,
    MessageKey::SourcesEmptyBody,
    MessageKey::SourcesAddFolder,
    MessageKey::SourcesStatusActive,
    MessageKey::SourcesStatusPaused,
    MessageKey::SourcesStatusMissing,
    MessageKey::IndexingTitle,
    MessageKey::IndexingIdle,
    MessageKey::IndexingHealthIndexed,
    MessageKey::IndexingHealthStale,
    MessageKey::IndexingHealthFailed,
    MessageKey::IndexingHealthQueued,
    MessageKey::StorageTitle,
    MessageKey::StorageIntro,
    MessageKey::StorageSafeCleanupHeading,
    MessageKey::StorageClearSnippets,
    MessageKey::StorageClearSearchCache,
    MessageKey::StorageDangerHeading,
    MessageKey::StorageResetCatalog,
    MessageKey::StorageResetWarning,
    MessageKey::StorageGroupSearchIndex,
    MessageKey::StorageGroupModels,
    MessageKey::StorageGroupCaches,
    MessageKey::ModelsTitle,
    MessageKey::ModelsEmbeddingRole,
    MessageKey::ModelsRerankerRole,
    MessageKey::ModelsStatusAvailable,
    MessageKey::ModelsStatusMissing,
    MessageKey::ModelsKeywordOnlyHint,
    MessageKey::SettingsTitle,
    MessageKey::SettingsLanguageHeading,
    MessageKey::SettingsThemeHeading,
    MessageKey::ThemeSystem,
    MessageKey::ThemeLight,
    MessageKey::ThemeDark,
    MessageKey::ThemeHighContrastLight,
    MessageKey::ThemeHighContrastDark,
    MessageKey::SettingsTextScaleHeading,
    MessageKey::TextScaleDefault,
    MessageKey::TextScaleLarge,
    MessageKey::TextScaleLarger,
    MessageKey::SettingsReduceMotion,
    MessageKey::SettingsReduceMotionHint,
    MessageKey::SettingsCvdNote,
    MessageKey::SettingsPrivacyHeading,
    MessageKey::SettingsPrivacyLocalOnly,
    MessageKey::SettingsAdvancedHeading,
    MessageKey::SettingsAdvancedOn,
    MessageKey::SettingsAdvancedOff,
    MessageKey::SettingsAdvancedHint,
    MessageKey::NoticeDownloadFailTitle,
    MessageKey::NoticeDownloadFailBody,
    MessageKey::NoticeFolderFailTitle,
    MessageKey::NoticeFolderFailBody,
    MessageKey::NoticeSearchFailTitle,
    MessageKey::NoticeSearchFailBody,
    MessageKey::NoticeFilesMissingTitle,
    MessageKey::NoticeFilesMissingBody,
    MessageKey::NoticeFolderAddedTitle,
    MessageKey::NoticeFolderAddedBody,
    MessageKey::NoticeSearchReadyTitle,
    MessageKey::NoticeSearchReadyBody,
    MessageKey::NoticePreviewsClearedTitle,
    MessageKey::NoticePreviewsClearedBody,
    MessageKey::NoticeActionTryAgain,
    MessageKey::NoticeActionChooseFolder,
    MessageKey::NoticeSensitiveSourceTitle,
    MessageKey::NoticeSensitiveSourceBody,
    MessageKey::NoticeDismiss,
    MessageKey::Cancel,
    MessageKey::Confirm,
];
