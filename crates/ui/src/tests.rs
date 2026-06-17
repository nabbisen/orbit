//! orbok-ui tests. Catalog completeness is compile-time by construction
//! (exhaustive matches); these tests validate runtime properties from
//! RFC-031 §9 and the headless state transitions.

use crate::i18n::{Locale, MessageKey, files_indexed, source_summary, tr};
use crate::state::{AppState, Message, ViewId};

const ALL_KEYS: &[MessageKey] = &[
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
    MessageKey::ModelsTitle,
    MessageKey::ModelsEmbeddingRole,
    MessageKey::ModelsRerankerRole,
    MessageKey::ModelsStatusAvailable,
    MessageKey::ModelsStatusMissing,
    MessageKey::ModelsKeywordOnlyHint,
    MessageKey::SettingsTitle,
    MessageKey::SettingsLanguageHeading,
    MessageKey::SettingsPrivacyHeading,
    MessageKey::SettingsPrivacyLocalOnly,
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
    MessageKey::NoticeDismiss,
    MessageKey::Cancel,
    MessageKey::Confirm,
];

// RFC-031 §9: every key resolves to a non-empty string in every locale.
#[test]
fn all_messages_non_empty_in_all_locales() {
    for locale in Locale::ALL {
        for key in ALL_KEYS {
            assert!(
                !tr(*locale, *key).is_empty(),
                "{locale:?} {key:?} is empty"
            );
        }
    }
}

// RFC-031 §9: locales actually differ (a copy-pasted catalog is a bug).
#[test]
fn locales_differ_for_translatable_keys() {
    let differing = ALL_KEYS
        .iter()
        .filter(|key| tr(Locale::En, **key) != tr(Locale::Ja, **key))
        .count();
    assert!(differing > ALL_KEYS.len() / 2, "catalogs are suspiciously identical");
}

// RFC-031 §5.3: parameterized messages localize.
#[test]
fn parameterized_messages_localize() {
    assert!(files_indexed(Locale::En, 3).contains("3 files"));
    assert!(files_indexed(Locale::Ja, 3).contains('3'));
    assert_ne!(
        source_summary(Locale::En, 1, 2, 3),
        source_summary(Locale::Ja, 1, 2, 3)
    );
}

// Locale persistence round-trip ("ui.locale" setting format).
#[test]
fn locale_setting_round_trip() {
    for locale in Locale::ALL {
        assert_eq!(Locale::parse(locale.as_str()), Some(*locale));
    }
    assert_eq!(Locale::parse("xx"), None);
}

// Headless state transitions (RFC-027: view models testable without a
// display).
#[test]
fn state_transitions() {
    let mut state = AppState::default();
    assert_eq!(state.active_view, ViewId::Search);

    state.update(&Message::Switch(ViewId::Storage));
    assert_eq!(state.active_view, ViewId::Storage);

    state.update(&Message::QueryChanged("  token expiry ".into()));
    state.update(&Message::SubmitSearch);
    assert_eq!(state.last_query.as_deref(), Some("token expiry"));

    state.update(&Message::QueryChanged("   ".into()));
    state.update(&Message::SubmitSearch);
    // Blank query does not clobber the last submitted one.
    assert_eq!(state.last_query.as_deref(), Some("token expiry"));

    state.update(&Message::SetLocale(Locale::Ja));
    assert_eq!(state.locale, Locale::Ja);
}

// Sidebar covers all six pages in design order (GUI design §2.2).
#[test]
fn navigation_order_is_search_first() {
    assert_eq!(ViewId::ALL.len(), 6);
    assert_eq!(ViewId::ALL[0], ViewId::Search);
    assert_eq!(ViewId::ALL[5], ViewId::Settings);
}

mod smoke_views;

// UX review §7: failures surface a visible notice; success clears it.
#[test]
fn failures_surface_notice_success_clears_it() {
    use crate::notice::UserNotice;
    let mut state = AppState::default();
    assert!(state.notice.is_none(), "no notice initially");

    // A search error must produce a visible, friendly notice.
    state.update(&Message::SearchError("backend exploded".into()));
    assert_eq!(state.notice, Some(UserNotice::SearchDidNotFinish));

    // A successful result must clear the notice.
    state.update(&Message::SearchResultsReady(vec![]));
    assert!(state.notice.is_none(), "successful search clears the notice");

    // ClearNotice dismisses any active notice.
    state.update(&Message::ShowNotice(UserNotice::DownloadDidNotFinish));
    assert!(state.notice.is_some());
    state.update(&Message::ClearNotice);
    assert!(state.notice.is_none(), "ClearNotice dismisses the notice");
}

// UX review: a problem notice offers a recovery action; a confirmation does not.
#[test]
fn problem_notices_offer_action_confirmations_do_not() {
    use crate::notice::UserNotice;
    let loc = Locale::En;
    assert!(UserNotice::DownloadDidNotFinish.action(loc).is_some());
    assert!(UserNotice::SearchDidNotFinish.action(loc).is_some());
    assert!(UserNotice::FolderAdded.action(loc).is_none());
    assert!(UserNotice::SearchReady.action(loc).is_none());
    assert!(UserNotice::DownloadDidNotFinish.is_problem());
    assert!(!UserNotice::FolderAdded.is_problem());
}
