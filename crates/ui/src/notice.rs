//! User-facing notices (UX review §7): friendly, actionable messages that
//! replace silent failures and raw error strings.
//!
//! Lower layers (download, scanner, search) produce technical errors. The UI
//! must never show those directly. Instead they are mapped to a [`UserNotice`]
//! with a plain title, an explanation, and a suggested next action.

use crate::i18n::{tr, Locale, MessageKey};

/// A friendly, actionable message shown to the user. Covers both problems
/// (download failed) and confirmations (folder added).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserNotice {
    // ── Problems ──────────────────────────────────────────────────────
    DownloadDidNotFinish,
    FolderCouldNotBeAdded,
    SearchDidNotFinish,
    FilesMovedOrMissing,
    // ── Confirmations ─────────────────────────────────────────────────
    FolderAdded,
    SearchReady,
    PreviewsCleared,
}

impl UserNotice {
    /// Whether this notice reports a problem (vs. a success confirmation).
    /// The view can use this to choose tone, but never relies on colour alone.
    pub fn is_problem(&self) -> bool {
        matches!(
            self,
            Self::DownloadDidNotFinish
                | Self::FolderCouldNotBeAdded
                | Self::SearchDidNotFinish
                | Self::FilesMovedOrMissing
        )
    }

    pub fn title(&self, locale: Locale) -> &'static str {
        let key = match self {
            Self::DownloadDidNotFinish => MessageKey::NoticeDownloadFailTitle,
            Self::FolderCouldNotBeAdded => MessageKey::NoticeFolderFailTitle,
            Self::SearchDidNotFinish => MessageKey::NoticeSearchFailTitle,
            Self::FilesMovedOrMissing => MessageKey::NoticeFilesMissingTitle,
            Self::FolderAdded => MessageKey::NoticeFolderAddedTitle,
            Self::SearchReady => MessageKey::NoticeSearchReadyTitle,
            Self::PreviewsCleared => MessageKey::NoticePreviewsClearedTitle,
        };
        tr(locale, key)
    }

    pub fn body(&self, locale: Locale) -> &'static str {
        let key = match self {
            Self::DownloadDidNotFinish => MessageKey::NoticeDownloadFailBody,
            Self::FolderCouldNotBeAdded => MessageKey::NoticeFolderFailBody,
            Self::SearchDidNotFinish => MessageKey::NoticeSearchFailBody,
            Self::FilesMovedOrMissing => MessageKey::NoticeFilesMissingBody,
            Self::FolderAdded => MessageKey::NoticeFolderAddedBody,
            Self::SearchReady => MessageKey::NoticeSearchReadyBody,
            Self::PreviewsCleared => MessageKey::NoticePreviewsClearedBody,
        };
        tr(locale, key)
    }

    /// Suggested next-action label, if the notice offers a recovery action.
    /// Confirmations return `None` (they are dismissed, not acted upon).
    pub fn action(&self, locale: Locale) -> Option<&'static str> {
        let key = match self {
            Self::DownloadDidNotFinish | Self::SearchDidNotFinish => MessageKey::NoticeActionTryAgain,
            Self::FolderCouldNotBeAdded => MessageKey::NoticeActionChooseFolder,
            Self::FilesMovedOrMissing => MessageKey::NoticeActionChooseFolder,
            Self::FolderAdded | Self::SearchReady | Self::PreviewsCleared => return None,
        };
        Some(tr(locale, key))
    }
}
