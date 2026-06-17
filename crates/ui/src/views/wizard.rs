//! Wizard views: model setup, download progress, file-check, and ready pages.
//!
//! Design (GUI spec §6 and RFC-012): The wizard runs at every launch when the
//! embedding model is missing or invalid. It has four pages:
//!
//! 1. **Setup** — shown on `NotConfigured` or `FileMissing`. Primary action is
//!    "Download from HuggingFace"; secondary is "Locate existing files".
//! 2. **Downloading** — progress bar while the model is being fetched.
//! 3. **Checked** — shows per-file ✓/✗ after the user locates files manually.
//! 4. **Ready** — confirmation that the model is loaded; wizard dismisses.

use lucide_icons::iced as icons;
use crate::i18n::{Locale, MessageKey, tr};
use crate::state::{AppState, Message, WizardFileCheck, WizardState};
use iced::widget::{button, column, container, progress_bar, row, text, text_input};
use iced::{Element, Length};

/// Dispatch to the correct wizard page.
pub fn wizard_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    match state.wizard.as_ref().expect("wizard_view called without active wizard") {
        WizardState::NotConfigured => page_setup(locale, state, None),
        WizardState::FileMissing { previous_dir, checks } => {
            page_setup(locale, state, Some((previous_dir.as_str(), checks.as_slice())))
        }
        WizardState::Downloading {
            current_file,
            bytes,
            total,
            files_done,
            files_total,
            ..
        } => page_downloading(locale, current_file, *bytes, *total, *files_done, *files_total),
        WizardState::Checked { model_dir, checks, all_ok } => {
            page_checked(locale, state, model_dir, checks, *all_ok)
        }
        WizardState::Ready { model_dir } => page_ready(locale, model_dir),
    }
}

// ── Page: setup ──────────────────────────────────────────────────────

fn page_setup<'a>(
    locale: Locale,
    state: &'a AppState,
    missing: Option<(&'a str, &'a [WizardFileCheck])>,
) -> Element<'a, Message> {
    let mut col = column![
        text(tr(locale, MessageKey::WizardTitleNotConfigured))
            .size(22),
        text(tr(locale, MessageKey::WizardBodyNotConfigured))
            .size(13),
    ]
    .spacing(8);

    // ── Primary action: Download ──────────────────────────────────────
    let download_card = container(
        column![
            row![
                icons::icon_download().size(16),
                text(tr(locale, MessageKey::WizardDownloadAction)).size(14),
            ]
            .spacing(6),
            text("multilingual-e5-small · Apache 2.0 · ~93 MB · 100+ languages")
                .size(11),
            button(
                row![
                    icons::icon_download().size(13),
                    text(tr(locale, MessageKey::WizardDownloadAction)).size(13),
                ]
                .spacing(4),
            )
            .on_press(Message::DownloadModel),
        ]
        .spacing(6),
    )
    .padding(12);
    col = col.push(download_card);

    // ── Separator ────────────────────────────────────────────────────
    col = col.push(text("— or —").size(11));

    // ── Secondary action: locate existing files ───────────────────────
    col = col.push(
        text(tr(locale, MessageKey::WizardBodyFileMissing)).size(12),
    );

    // Show previous path hint when files were missing.
    if let Some((prev_dir, checks)) = missing {
        col = col.push(text(prev_dir).size(11));
        for fc in checks {
            let (icon, note) = if fc.found { ("✓", "") } else { ("✗", "  ← missing") };
            col = col.push(text(format!("{icon}  {}{note}", fc.relative_path)).size(11));
        }
    }

    let path_input = text_input(
        tr(locale, MessageKey::WizardPathPlaceholder),
        &state.wizard_path_input,
    )
    .on_input(Message::WizardPathChanged)
    .on_submit(Message::WizardValidate)
    .padding(8);

    col = col.push(
        row![
            container(path_input).width(Length::Fill),
            button(
                row![
                    icons::icon_folder_open().size(13),
                    text(tr(locale, MessageKey::WizardActionValidate)).size(13),
                ]
                .spacing(4),
            )
            .on_press(Message::WizardValidate),
        ]
        .spacing(8),
    );

    // ── Tertiary action: skip ─────────────────────────────────────────
    col = col.push(
        button(text(tr(locale, MessageKey::WizardActionSkip)).size(12))
            .on_press(Message::WizardSkip),
    );

    container(col.spacing(10))
        .padding(iced::Padding::from([32.0, 40.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Page: download progress ──────────────────────────────────────────

fn page_downloading<'a>(
    locale: Locale,
    current_file: &'a str,
    bytes: u64,
    total: Option<u64>,
    files_done: u32,
    files_total: u32,
) -> Element<'a, Message> {
    let overall_label = format!("File {}/{}", files_done + 1, files_total);

    // Progress fraction for the current file (0.0 – 1.0).
    let frac: f32 = match total {
        Some(t) if t > 0 => (bytes as f32 / t as f32).min(1.0),
        _ => 0.0,
    };

    let bytes_label = if let Some(t) = total {
        format!(
            "{} / {}",
            human_bytes(bytes),
            human_bytes(t),
        )
    } else {
        human_bytes(bytes)
    };

    let pct_label = if total.is_some() {
        format!("  ({:.0}%)", frac * 100.0)
    } else {
        String::new()
    };

    let col = column![
        row![
            icons::icon_download().size(16),
            text(tr(locale, MessageKey::WizardDownloadProgress)).size(20),
        ]
        .spacing(6),
        text("multilingual-e5-small · Apache 2.0")
            .size(11),
        text(overall_label).size(12),
        text(format!("↓  {current_file}")).size(13),
        progress_bar(0.0..=1.0, frac),
        text(format!("{bytes_label}{pct_label}")).size(11),
    ]
    .spacing(10);

    container(col)
        .padding(iced::Padding::from([32.0, 40.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Page: file check results ─────────────────────────────────────────

fn page_checked<'a>(
    locale: Locale,
    state: &'a AppState,
    model_dir: &'a str,
    checks: &'a [WizardFileCheck],
    all_ok: bool,
) -> Element<'a, Message> {
    let mut col = column![
        text(tr(locale, MessageKey::WizardTitleValidating)).size(20),
        text(model_dir).size(11),
    ]
    .spacing(8);

    for fc in checks {
        let (icon, style) = if fc.found { ("✓", "") } else { ("✗", "  ← missing") };
        let size_info = fc.size_mb.map(|m| format!("  ({m} MB)")).unwrap_or_default();
        col = col.push(
            text(format!("{icon}  {}{size_info}{style}", fc.relative_path)).size(12),
        );
    }

    if all_ok {
        col = col.push(
            button(
                row![
                    icons::icon_check_circle().size(13),
                    text(tr(locale, MessageKey::WizardActionUseModel)).size(13),
                ]
                .spacing(4),
            )
            .on_press(Message::WizardAccept),
        );
    } else {
        col = col.push(text(tr(locale, MessageKey::WizardBodyFileMissing)).size(12));
        let path_input = text_input(
            tr(locale, MessageKey::WizardPathPlaceholder),
            &state.wizard_path_input,
        )
        .on_input(Message::WizardPathChanged)
        .on_submit(Message::WizardValidate)
        .padding(8);
        col = col.push(
            row![
                container(path_input).width(Length::Fill),
                button(
                    row![
                        icons::icon_scan_eye().size(13),
                        text(tr(locale, MessageKey::WizardActionValidate)).size(13),
                    ]
                    .spacing(4),
                )
                .on_press(Message::WizardValidate),
            ]
            .spacing(8),
        );
    }

    col = col.push(
        row![
            button(text("← Back").size(12)).on_press(Message::WizardBack),
            button(text(tr(locale, MessageKey::WizardActionSkip)).size(12))
                .on_press(Message::WizardSkip),
        ].spacing(8),
    );

    container(col.spacing(10))
        .padding(iced::Padding::from([32.0, 40.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Page: ready ───────────────────────────────────────────────────────

fn page_ready<'a>(locale: Locale, model_dir: &'a str) -> Element<'a, Message> {
    let col = column![
        row![
            icons::icon_check_circle().size(18),
            text(tr(locale, MessageKey::WizardTitleReady)).size(20),
        ]
        .spacing(6),
        text(model_dir).size(11),
        text(tr(locale, MessageKey::WizardReadyBody)).size(13),
        button(
            row![
                icons::icon_check_circle().size(13),
                text(tr(locale, MessageKey::WizardActionUseModel)).size(13),
            ]
            .spacing(4),
        )
        .on_press(Message::WizardAccept),
    ]
    .spacing(10);

    container(col)
        .padding(iced::Padding::from([32.0, 40.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── helpers ───────────────────────────────────────────────────────────

fn human_bytes(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1} MB", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0} KB", n as f64 / 1_000.0)
    } else {
        format!("{n} B")
    }
}
