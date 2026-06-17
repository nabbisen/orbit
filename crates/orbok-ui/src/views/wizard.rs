//! Startup wizard view (design §wizard).
//!
//! Shown instead of the normal shell when `AppState::wizard` is `Some`.
//! Four states map to the designed pages:
//! - `NotConfigured` → first-launch setup
//! - `FileMissing`   → model was registered, file is gone
//! - `Checked`       → path submitted, file checks shown inline
//! - `Ready`         → all files valid, ready to continue

use crate::i18n::{MessageKey, Locale, tr};
use crate::state::{AppState, Message, WizardFileCheck, WizardState};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Padding};

/// Render the appropriate wizard page based on `state.wizard`.
pub fn wizard_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let wizard = match &state.wizard {
        Some(w) => w,
        None => return text("").into(),
    };
    let inner = match wizard {
        WizardState::NotConfigured => page_input(locale, state, None),
        WizardState::FileMissing { previous_dir } => {
            page_input(locale, state, Some(previous_dir))
        }
        WizardState::Checked { model_dir, checks, all_ok } => {
            page_checked(locale, state, model_dir, checks, *all_ok)
        }
        WizardState::Ready { model_dir } => page_ready(locale, model_dir),
    };
    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::from([40.0, 32.0]))
        .into()
}

// ── Page: input (NotConfigured or FileMissing) ────────────────────────

fn page_input<'a>(
    locale: Locale,
    state: &'a AppState,
    previous_dir: Option<&'a str>,
) -> Element<'a, Message> {
    let (title_key, body_key) = if previous_dir.is_some() {
        (MessageKey::WizardTitleFileMissing, MessageKey::WizardBodyFileMissing)
    } else {
        (MessageKey::WizardTitleNotConfigured, MessageKey::WizardBodyNotConfigured)
    };

    let mut col = column![
        text(tr(locale, title_key)).size(22),
        text(tr(locale, body_key)).size(13),
    ]
    .spacing(10);

    // Show the previous path struck-through when file is missing.
    if let Some(prev) = previous_dir {
        col = col.push(
            column![
                text(tr(locale, MessageKey::WizardPreviousPathLabel)).size(12),
                text(format!("  {prev}")).size(11),
            ]
            .spacing(2),
        );
    } else {
        // Show what files are needed + download hint.
        col = col.push(
            column![
                text(tr(locale, MessageKey::WizardFilesNeededLabel)).size(12),
                text("  onnx/model.onnx").size(11),
                text("  tokenizer.json").size(11),
                text(tr(locale, MessageKey::WizardDownloadHint)).size(11),
            ]
            .spacing(2),
        );
    }

    col = col.push(
        text_input(
            tr(locale, MessageKey::WizardPathInputPlaceholder),
            &state.wizard_path_input,
        )
        .on_input(Message::WizardPathChanged)
        .on_submit(Message::WizardValidate)
        .padding(8),
    );

    col = col.push(
        row![
            button(text(tr(locale, MessageKey::WizardActionValidate)).size(13))
                .on_press(Message::WizardValidate),
        ]
        .spacing(8),
    );

    col = col.push(
        button(text(tr(locale, MessageKey::WizardActionSkip)).size(12))
            .on_press(Message::WizardSkip),
    );

    col.into()
}

// ── Page: file check results ─────────────────────────────────────────

fn page_checked<'a>(
    locale: Locale,
    state: &'a AppState,
    model_dir: &'a str,
    checks: &'a [WizardFileCheck],
    all_ok: bool,
) -> Element<'a, Message> {
    let title = tr(locale, MessageKey::WizardTitleValidating);
    let mut col = column![
        text(title).size(22),
        text(model_dir).size(11),
    ]
    .spacing(10);

    for check in checks {
        let status = if check.found {
            let mb = check.size_mb.map(|m| format!("  {m:.1} MB")).unwrap_or_default();
            format!(
                "✓ {}{}  {}",
                check.relative_path,
                mb,
                tr(locale, MessageKey::WizardValidationOk)
            )
        } else {
            format!(
                "✗ {}  {}",
                check.relative_path,
                tr(locale, MessageKey::WizardValidationFail)
            )
        };
        col = col.push(text(status).size(12));
    }

    if all_ok {
        col = col.push(
            button(text(tr(locale, MessageKey::WizardActionUseModel)).size(13))
                .on_press(Message::WizardAccept),
        );
    } else {
        // Allow re-entering path if checks failed.
        col = col.push(
            text_input(
                tr(locale, MessageKey::WizardPathInputPlaceholder),
                &state.wizard_path_input,
            )
            .on_input(Message::WizardPathChanged)
            .on_submit(Message::WizardValidate)
            .padding(8),
        );
        col = col.push(
            button(text(tr(locale, MessageKey::WizardActionValidate)).size(13))
                .on_press(Message::WizardValidate),
        );
    }

    col = col.push(
        button(text(tr(locale, MessageKey::WizardActionSkip)).size(12))
            .on_press(Message::WizardSkip),
    );

    col.into()
}

// ── Page: ready ──────────────────────────────────────────────────────

fn page_ready<'a>(locale: Locale, model_dir: &'a str) -> Element<'a, Message> {
    column![
        text(tr(locale, MessageKey::WizardTitleReady)).size(22),
        text(tr(locale, MessageKey::WizardReadyBody)).size(13),
        text(model_dir).size(11),
        button(text(tr(locale, MessageKey::WizardActionContinue)).size(13))
            .on_press(Message::WizardAccept),
    ]
    .spacing(10)
    .into()
}
