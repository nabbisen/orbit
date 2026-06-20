//! English catalog (RFC-031). Exhaustive over [`MessageKey`].

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    use MessageKey::*;
    match key {
        AppTitle => "orbok",
        LocalOnlyBadge => "Local Only",
        NavSearch => "Search",
        NavSources => "Sources",
        NavIndexing => "Preparing",
        NavStorage => "Storage",
        NavModels => "Models",
        NavAi => "AI",
        NavSettings => "Settings",
        SearchPlaceholder => "Search local documents...",
        SearchButton => "Search",
        SearchNoSourcesTitle => "Nothing to search yet",
        SearchNoSourcesBody => "Add a folder or file so orbok can build a local search index.",
        SearchAddSource => "Add Source",
        SearchNoResults => "No results found",
        SearchKeywordOnlyNotice => "Search by meaning is not set up yet. Basic search still works.",
        SourcesTitle => "Sources",
        SourcesEmptyTitle => "No sources added",
        SourcesEmptyBody => {
            "Add folders or files that orbok is allowed to search. \
             orbok will not scan your entire computer automatically."
        }
        SourcesAddFolder => "Add Folder",
        SourcesStatusActive => "Active",
        SourcesStatusPaused => "Paused",
        SourcesStatusMissing => "Missing",
        IndexingTitle => "Preparing search",
        IndexingIdle => "Search is ready",
        IndexingHealthIndexed => "Indexed",
        IndexingHealthStale => "Stale",
        IndexingHealthFailed => "Failed",
        IndexingHealthQueued => "Queued",
        StorageTitle => "Storage",
        StorageIntro => "See what orbok stores and clean up safely.",
        StorageGroupSearchIndex => "Search data",
        StorageGroupModels => "Search helper",
        StorageGroupCaches => "Temporary previews",
        StorageSafeCleanupHeading => "Safe cleanup",
        StorageClearSnippets => "Clear temporary previews",
        StorageClearSearchCache => "Clear old search results",
        StorageDangerHeading => "Dangerous",
        StorageResetCatalog => "Reset saved app data...",
        StorageResetWarning => {
            "This removes registered sources and all indexes. \
             Your source files are never deleted."
        }
        ModelsTitle => "Models",
        ModelsEmbeddingRole => "Embedding",
        ModelsRerankerRole => "Reranker",
        ModelsStatusAvailable => "Available",
        ModelsStatusMissing => "Missing",
        ModelsKeywordOnlyHint => {
            "Basic search still works. Add a search helper to also \
             search by meaning."
        }
        SettingsTitle => "Settings",
        SettingsLanguageHeading => "Language",
        SettingsPrivacyHeading => "Privacy",
        SettingsAdvancedHeading => "Advanced view",
        SettingsAdvancedOn => "Advanced view: On",
        SettingsAdvancedOff => "Advanced view: Off",
        SettingsAdvancedHint => "Show technical detail in search results, indexing, and storage.",
        SettingsPrivacyLocalOnly => "Documents are processed on this computer only.",
        SearchModeLabel => "Mode",
        SearchModeAuto => "Auto",
        SearchModeExact => "Exact",
        SearchModeConceptual => "Conceptual",
        SearchModeFast => "Fast",
        BadgeKeyword => "Keyword",
        BadgeSemantic => "Semantic",
        BadgeFused => "Fused",
        WizardTitleNotConfigured => "Set up search by meaning",
        WizardTitleFileMissing => "Embedding model not found",
        WizardTitleValidating => "Checking model folder",
        WizardTitleReady => "Embedding model ready",
        WizardBodyNotConfigured => {
            "Keyword search is ready. To also search by meaning,              orbok needs a local AI model on this computer.              No files are uploaded — inference runs locally."
        }
        WizardBodyFileMissing => {
            "The model folder is no longer at its expected location.              This can happen when a drive is disconnected or files are moved."
        }
        WizardFilesNeededLabel => "Required files in the folder:",
        WizardDownloadHint => "Download: huggingface-cli download intfloat/multilingual-e5-small",
        WizardPathInputPlaceholder => "Path to model folder (e.g. ~/models/multilingual-e5-small)",
        WizardActionLocate => "Locate model folder",
        WizardActionValidate => "Validate",
        WizardActionUseModel => "Use this model",
        WizardActionContinue => "Continue to orbok",
        WizardPathPlaceholder => "Folder path…",
        WizardDownloadAction => "Download from HuggingFace",
        WizardDownloadProgress => "Downloading model…",
        WizardActionSkip => "Skip — use keyword search only",
        WizardPreviousPathLabel => "Last known path",
        WizardValidationOk => "found",
        WizardValidationFail => "not found",
        WizardReadyBody => "Semantic search is now available.",
        NoticeDownloadFailTitle => "Download did not finish",
        NoticeDownloadFailBody => {
            "We could not finish the download. Please check your \
             connection and try again."
        }
        NoticeFolderFailTitle => "Folder was not added",
        NoticeFolderFailBody => {
            "We could not add that folder. Please choose another folder \
             or check that you can open it."
        }
        NoticeSearchFailTitle => "Search did not finish",
        NoticeSearchFailBody => "Something went wrong while searching. Please try again.",
        NoticeFilesMissingTitle => "Files may have moved",
        NoticeFilesMissingBody => {
            "Some files are no longer where orbok expected them. This can \
             happen if a drive was disconnected or files were moved."
        }
        NoticeFolderAddedTitle => "Folder added",
        NoticeFolderAddedBody => "orbok is preparing your search now.",
        NoticeSearchReadyTitle => "Search is ready",
        NoticeSearchReadyBody => "Your files are ready to search.",
        NoticePreviewsClearedTitle => "Temporary previews cleared",
        NoticePreviewsClearedBody => "Freed up space. Your files are untouched.",
        NoticeActionTryAgain => "Try again",
        NoticeActionChooseFolder => "Choose another folder",
        SettingsThemeHeading => "Theme",
        ThemeSystem => "Follow system",
        ThemeLight => "Light",
        ThemeDark => "Dark",
        ThemeHighContrastLight => "High contrast (light)",
        ThemeHighContrastDark => "High contrast (dark)",
        SettingsTextScaleHeading => "Text size",
        TextScaleDefault => "Default",
        TextScaleLarge => "Large",
        TextScaleLarger => "Larger",
        SettingsReduceMotion => "Reduce motion",
        SettingsReduceMotionHint => "Fewer animations and transitions.",
        SettingsCvdNote => "Status colors are always shown with a label and an icon, so they stay clear for every kind of color vision.",
        NoticeSensitiveSourceTitle => "This folder may contain private files",
        NoticeSensitiveSourceBody => {
            "It may include SSH keys, browser profiles, or other sensitive data. The folder was added. Remove it if you did not intend to search it."
        }
        NoticeDismiss => "Dismiss",
        Cancel => "Cancel",
        Confirm => "Confirm",
    }
}
