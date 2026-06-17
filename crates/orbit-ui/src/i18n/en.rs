//! English catalog (RFC-031). Exhaustive over [`MessageKey`].

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    use MessageKey::*;
    match key {
        AppTitle => "orbit",
        LocalOnlyBadge => "Local Only",
        NavSearch => "Search",
        NavSources => "Sources",
        NavIndexing => "Indexing",
        NavStorage => "Storage",
        NavModels => "Models",
        NavSettings => "Settings",
        SearchPlaceholder => "Search local documents...",
        SearchButton => "Search",
        SearchNoSourcesTitle => "Nothing to search yet",
        SearchNoSourcesBody => "Add a folder or file so orbit can build a local search index.",
        SearchAddSource => "Add Source",
        SearchNoResults => "No results found",
        SearchKeywordOnlyNotice => {
            "Semantic search is unavailable. Keyword search still works."
        }
        SourcesTitle => "Sources",
        SourcesEmptyTitle => "No sources added",
        SourcesEmptyBody => {
            "Add folders or files that orbit is allowed to search. \
             orbit will not scan your entire computer automatically."
        }
        SourcesAddFolder => "Add Folder",
        SourcesStatusActive => "Active",
        SourcesStatusPaused => "Paused",
        SourcesStatusMissing => "Missing",
        IndexingTitle => "Indexing",
        IndexingIdle => "Index is up to date",
        IndexingHealthIndexed => "Indexed",
        IndexingHealthStale => "Stale",
        IndexingHealthFailed => "Failed",
        IndexingHealthQueued => "Queued",
        StorageTitle => "Storage",
        StorageIntro => "See what orbit stores and clean up safely.",
        StorageSafeCleanupHeading => "Safe cleanup",
        StorageClearSnippets => "Clear temporary snippets",
        StorageClearSearchCache => "Clear expired search cache",
        StorageDangerHeading => "Dangerous",
        StorageResetCatalog => "Reset catalog...",
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
            "Keyword search still works. Install or locate an embedding \
             model to enable conceptual search."
        }
        SettingsTitle => "Settings",
        SettingsLanguageHeading => "Language",
        SettingsPrivacyHeading => "Privacy",
        SettingsPrivacyLocalOnly => "Documents are processed on this computer only.",
        Cancel => "Cancel",
        Confirm => "Confirm",
    }
}
