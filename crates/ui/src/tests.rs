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
    MessageKey::SettingsThemeHeading,
    MessageKey::ThemeSystem,
    MessageKey::ThemeLight,
    MessageKey::ThemeDark,
    MessageKey::ThemeHighContrastLight,
    MessageKey::ThemeHighContrastDark,
    MessageKey::NoticeSensitiveSourceTitle,
    MessageKey::NoticeSensitiveSourceBody,
    MessageKey::NoticeDismiss,
    MessageKey::Cancel,
    MessageKey::Confirm,
];

// RFC-031 §9: every key resolves to a non-empty string in every locale.
#[test]
fn all_messages_non_empty_in_all_locales() {
    for locale in Locale::ALL {
        for key in ALL_KEYS {
            assert!(!tr(*locale, *key).is_empty(), "{locale:?} {key:?} is empty");
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
    assert!(
        differing > ALL_KEYS.len() / 2,
        "catalogs are suspiciously identical"
    );
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
    assert!(
        state.notice.is_none(),
        "successful search clears the notice"
    );

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

// RFC-031 §3: auto locale resolves Japanese OS environments to ja.
#[test]
fn locale_from_env_detects_japanese() {
    let prev = std::env::var("LANG").ok();
    // SAFETY: single-threaded test; no other threads are reading LANG.
    unsafe {
        std::env::set_var("LANG", "ja_JP.UTF-8");
    }
    let detected = Locale::from_env();
    unsafe {
        match prev {
            Some(v) => std::env::set_var("LANG", v),
            None => std::env::remove_var("LANG"),
        }
    }
    assert_eq!(detected, Some(Locale::Ja));
}

// RFC-031 §3: non-Japanese LANG falls through to English.
#[test]
fn locale_from_env_english_fallback() {
    let prev = std::env::var("LANG").ok();
    unsafe {
        std::env::set_var("LANG", "en_US.UTF-8");
    }
    let detected = Locale::from_env();
    unsafe {
        match prev {
            Some(v) => std::env::set_var("LANG", v),
            None => std::env::remove_var("LANG"),
        }
    }
    assert_eq!(detected, Some(Locale::En));
}

// snora 0.25 design migration: every notice maps to a tone, and problem
// notices use Danger/Warning while confirmations use Success/Info.
#[test]
fn notice_tone_mapping_is_consistent() {
    use crate::notice::UserNotice;
    use snora::design::Tone;

    // Problems must use attention-grabbing tones.
    assert_eq!(UserNotice::DownloadDidNotFinish.tone(), Tone::Danger);
    assert_eq!(UserNotice::FolderCouldNotBeAdded.tone(), Tone::Danger);
    assert_eq!(UserNotice::SearchDidNotFinish.tone(), Tone::Danger);
    assert_eq!(UserNotice::SensitiveSourceAdded.tone(), Tone::Warning);
    assert_eq!(UserNotice::FilesMovedOrMissing.tone(), Tone::Warning);

    // Confirmations must use positive/neutral tones.
    assert_eq!(UserNotice::FolderAdded.tone(), Tone::Success);
    assert_eq!(UserNotice::SearchReady.tone(), Tone::Success);
    assert_eq!(UserNotice::PreviewsCleared.tone(), Tone::Info);

    // Every problem notice is also flagged is_problem; tone agrees.
    for n in [
        UserNotice::DownloadDidNotFinish,
        UserNotice::FolderCouldNotBeAdded,
        UserNotice::SearchDidNotFinish,
    ] {
        assert!(n.is_problem());
        assert_eq!(n.tone(), Tone::Danger);
    }
}

// RFC-032: theme selection drives the active token preset.
#[test]
fn set_theme_swaps_token_preset() {
    use crate::theme::Theme;

    let mut state = AppState::default();
    assert_eq!(state.theme, Theme::System, "default theme is System");

    state.update(&Message::SetTheme(Theme::Dark));
    assert_eq!(state.theme, Theme::Dark);
    assert_eq!(state.tokens, snora::design::Tokens::dark());

    state.update(&Message::SetTheme(Theme::HighContrastLight));
    assert_eq!(state.theme, Theme::HighContrastLight);
    assert_eq!(state.tokens, snora::design::Tokens::high_contrast_light());

    state.update(&Message::SetTheme(Theme::Light));
    assert_eq!(state.tokens, snora::design::Tokens::light());
}

// RFC-032: every concrete theme maps to its snora preset, and the setting
// string round-trips.
#[test]
fn theme_tokens_and_string_roundtrip() {
    use crate::theme::Theme;

    let cases = [
        (Theme::Light, snora::design::Tokens::light()),
        (Theme::Dark, snora::design::Tokens::dark()),
        (
            Theme::HighContrastLight,
            snora::design::Tokens::high_contrast_light(),
        ),
        (
            Theme::HighContrastDark,
            snora::design::Tokens::high_contrast_dark(),
        ),
    ];
    for (theme, expected) in cases {
        assert_eq!(theme.tokens(), expected, "{theme:?} preset");
    }
    for theme in Theme::ALL {
        assert_eq!(
            Theme::parse(theme.as_str()),
            Some(*theme),
            "round-trip {theme:?}"
        );
    }
}

// RFC-032: the System theme resolves from the OS override env var. A concrete
// `ORBOK_THEME` wins; `system`/unset yield None (caller falls back to Light).
#[test]
fn theme_from_env_resolves_override() {
    use crate::theme::Theme;

    let prev = std::env::var("ORBOK_THEME").ok();
    // SAFETY: single-threaded test; ORBOK_THEME is read by no other test.
    unsafe {
        std::env::set_var("ORBOK_THEME", "dark");
    }
    assert_eq!(Theme::from_env(), Some(Theme::Dark));
    unsafe {
        std::env::set_var("ORBOK_THEME", "system");
    }
    assert_eq!(Theme::from_env(), None, "system override is not concrete");
    unsafe {
        std::env::remove_var("ORBOK_THEME");
    }
    assert_eq!(Theme::from_env(), None, "unset yields None");
    unsafe {
        match prev {
            Some(v) => std::env::set_var("ORBOK_THEME", v),
            None => std::env::remove_var("ORBOK_THEME"),
        }
    }
}
