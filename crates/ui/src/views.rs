//! Page view functions (GUI external design §7, §8–§12 wireframes).
//!
//! Each page is a plain function taking the view model and returning an
//! `Element`. Empty states follow the design's required empty-state set.
//!
//! Styling (RFC-032): no literal sizes, paddings, or colours — all values
//! come from `state.tokens` via [`crate::theme`] and `tokens.spacing`.
//!
//! Primitives (RFC-033): every card, button, badge, and progress element
//! routes through [`crate::components`]; snora is the sole primitive gateway.

pub mod wizard;
pub use wizard::wizard_view;

use crate::components::{self, health_cell, job_progress, result_card, source_card};
use crate::i18n::{Locale, MessageKey, files_indexed, search_result_count, source_summary, tr};
use crate::state::{AppState, Message};
use crate::theme::{self, Theme};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Padding};
use orbok_models::SearchCapability;
use snora::design::Tokens;
use snora::lucide;

/// A friendly, actionable notice card. Delegates to the snora Notice primitive
/// (already using design tokens via `crate::notice`).
fn friendly_notice<'a>(
    tokens: &'a Tokens,
    locale: Locale,
    notice: &crate::notice::UserNotice,
) -> Element<'a, Message> {
    use snora::design::notice::Notice;
    let mut builder = Notice::new(tokens, notice.tone(), notice.body(locale).to_string())
        .title(notice.title(locale).to_string());
    if let Some(action_label) = notice.action(locale) {
        builder = builder.action(action_label.to_string(), Message::ClearNotice);
    } else {
        builder = builder.dismiss(Message::ClearNotice);
    }
    builder.render()
}

fn page<'a>(tokens: &Tokens, content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(
        iced::widget::scrollable(
            container(content.spacing(tokens.spacing.md))
                .padding(Padding::from([tokens.spacing.xl, tokens.spacing.xxl]))
                .width(Length::Fill),
        )
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn heading<'a>(tokens: &Tokens, label: &'a str) -> iced::widget::Text<'a> {
    text(label.to_string()).size(theme::heading(tokens))
}

// ── Search view ──────────────────────────────────────────────────────────

pub fn search_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;

    let input = text_input(tr(locale, MessageKey::SearchPlaceholder), &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::SubmitSearch)
        .padding(tokens.spacing.sm);

    // Submit icon button via components::icon_primary; disabled while running.
    let submit = components::icon_primary(
        tokens,
        char::from(lucide::Search),
        13.0,
        tr(locale, MessageKey::SearchButton),
        (!state.search_running).then_some(Message::SubmitSearch),
    );

    let mut content = column![
        heading(tokens, tr(locale, MessageKey::NavSearch)),
        row![container(input).width(Length::Fill), submit].spacing(tokens.spacing.sm),
    ];

    if let Some(notice) = &state.notice {
        content = content.push(friendly_notice(tokens, locale, notice));
    }

    if state.show_advanced {
        content = content.push(
            row![
                text(tr(locale, MessageKey::SearchModeLabel)).size(theme::meta(tokens)),
                button(text(tr(locale, MessageKey::SearchModeAuto)).size(theme::meta(tokens)))
                    .on_press(Message::SetSearchMode(orbok_search::SearchMode::Auto)),
                button(text(tr(locale, MessageKey::SearchModeExact)).size(theme::meta(tokens)))
                    .on_press(Message::SetSearchMode(orbok_search::SearchMode::Exact)),
                button(
                    text(tr(locale, MessageKey::SearchModeConceptual)).size(theme::meta(tokens))
                )
                .on_press(Message::SetSearchMode(orbok_search::SearchMode::Conceptual)),
            ]
            .spacing(tokens.spacing.xs),
        );
    }

    if state.sources.is_empty() {
        content = content.push(
            column![
                text(tr(locale, MessageKey::SearchNoSourcesTitle)).size(theme::title(tokens)),
                text(tr(locale, MessageKey::SearchNoSourcesBody)).size(theme::body(tokens)),
                components::primary(
                    tokens,
                    tr(locale, MessageKey::SearchAddSource),
                    Some(Message::Switch(crate::state::ViewId::Sources)),
                ),
            ]
            .spacing(tokens.spacing.sm),
        );
    } else {
        if state.capability == SearchCapability::KeywordOnly {
            content = content.push(
                text(tr(locale, MessageKey::SearchKeywordOnlyNotice)).size(theme::meta(tokens)),
            );
        }
        if state.search_running {
            content = content.push(text("Searching…").size(theme::body(tokens)));
        } else if let Some(last) = &state.last_query {
            if state.search_results.is_empty() {
                content = content.push(
                    column![
                        text(tr(locale, MessageKey::SearchNoResults)).size(theme::body(tokens)),
                        text(format!("Query: {last}")).size(theme::meta(tokens)),
                    ]
                    .spacing(tokens.spacing.xs),
                );
            } else {
                content = content.push(
                    text(search_result_count(locale, state.search_results.len()))
                        .size(theme::meta(tokens)),
                );
                for (i, result) in state.search_results.iter().enumerate() {
                    let is_selected = state.selected_result == Some(i);
                    let title_raw = result.title.as_deref().unwrap_or(&result.display_path);
                    let title_str = if is_selected {
                        format!("▶  {title_raw}")
                    } else {
                        title_raw.to_string()
                    };
                    let snippet = result.snippet.as_deref().unwrap_or("(source unavailable)");
                    let heading_str = result.heading_path.as_deref().unwrap_or("");
                    content = content.push(result_card(
                        tokens,
                        title_str,
                        result.display_path.clone(),
                        heading_str.to_string(),
                        snippet.to_string(),
                        &result.badges,
                        state.show_advanced,
                        is_selected,
                        Message::SelectResult(i),
                    ));
                }
            }
        }
    }
    page(tokens, content)
}

// ── Sources view ─────────────────────────────────────────────────────────

pub fn sources_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;

    let add_btn = components::icon_secondary(
        tokens,
        char::from(lucide::FolderPlus),
        13.0,
        tr(locale, MessageKey::SourcesAddFolder),
        Some(Message::RequestAddSource),
    );
    let add_input = text_input("Or type a path manually…", &state.source_path_input)
        .on_input(Message::SourcePathChanged)
        .on_submit(Message::RequestAddSource)
        .padding(tokens.spacing.sm);

    let mut content = column![
        heading(tokens, tr(locale, MessageKey::SourcesTitle)),
        row![add_btn, container(add_input).width(Length::Fill)].spacing(tokens.spacing.sm),
        text("All sub-folders are scanned recursively.").size(theme::meta(tokens)),
    ];

    if let Some(notice) = &state.notice {
        content = content.push(friendly_notice(tokens, locale, notice));
    }
    if state.sources.is_empty() {
        content = content.push(
            column![
                text(tr(locale, MessageKey::SourcesEmptyTitle)).size(theme::title(tokens)),
                text(tr(locale, MessageKey::SourcesEmptyBody)).size(theme::body(tokens)),
            ]
            .spacing(tokens.spacing.sm),
        );
    } else {
        for card in &state.sources {
            let status_label = if card.active {
                tr(locale, MessageKey::SourcesStatusActive)
            } else {
                tr(locale, MessageKey::SourcesStatusPaused)
            };
            let summary = source_summary(locale, card.indexed, card.stale, card.failed);
            content = content.push(source_card(
                tokens,
                card.display_name.clone(),
                card.display_path.clone(),
                summary,
                status_label,
                Message::SourceRemoved(card.source_id.clone()),
            ));
        }
    }
    page(tokens, content)
}

// ── Indexing view ────────────────────────────────────────────────────────

pub fn indexing_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    let h = state.health;

    // Less is more: always show Indexed; show others only when non-zero or advanced.
    let mut cells = row![health_cell(
        tokens,
        tr(locale, MessageKey::IndexingHealthIndexed),
        h.indexed
    )]
    .spacing(tokens.spacing.sm);
    if h.queued > 0 || state.show_advanced {
        cells = cells.push(health_cell(tokens, tr(locale, MessageKey::IndexingHealthQueued), h.queued));
    }
    if h.stale > 0 || state.show_advanced {
        cells = cells.push(health_cell(tokens, tr(locale, MessageKey::IndexingHealthStale), h.stale));
    }
    if h.failed > 0 || state.show_advanced {
        cells = cells.push(health_cell(tokens, tr(locale, MessageKey::IndexingHealthFailed), h.failed));
    }

    // Show a progress row when indexing is active.
    let mut content = column![
        heading(tokens, tr(locale, MessageKey::IndexingTitle)),
        cells,
        text(if h.queued == 0 {
            tr(locale, MessageKey::IndexingIdle).to_string()
        } else {
            files_indexed(locale, h.indexed)
        })
        .size(theme::body(tokens)),
    ];

    if h.queued > 0 {
        content = content.push(job_progress(tokens, "Indexing…", None));
    }

    page(tokens, content)
}

// ── Storage view ─────────────────────────────────────────────────────────

pub fn storage_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;

    // Confirmation dialog (bespoke — no snora modal primitive yet).
    if state.confirm_reset {
        let content = column![
            text(tr(locale, MessageKey::StorageResetCatalog)).size(theme::title(tokens)),
            text(tr(locale, MessageKey::StorageResetWarning)).size(theme::body(tokens)),
            row![
                components::ghost(tokens, tr(locale, MessageKey::Cancel), Some(Message::CancelResetCatalog)),
                components::danger(tokens, tr(locale, MessageKey::StorageResetCatalog), Some(Message::ConfirmResetCatalog)),
            ]
            .spacing(tokens.spacing.md),
        ]
        .spacing(tokens.spacing.lg);
        return page(tokens, content);
    }

    let total_bytes: u64 = state.storage_rows.iter().map(|(_, b, _)| b).sum();
    let gib = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let mut breakdown = column![
        text(tr(locale, MessageKey::StorageTitle)).size(theme::heading(tokens)),
        text(tr(locale, MessageKey::StorageIntro)).size(theme::body(tokens)),
        text(format!("{gib:.3} GiB total")).size(theme::title(tokens)),
    ]
    .spacing(tokens.spacing.xs);

    if !state.storage_rows.is_empty() {
        if state.show_advanced {
            for (category, bytes, count) in &state.storage_rows {
                if *bytes > 0 || *count > 0 {
                    let mib = *bytes as f64 / (1024.0 * 1024.0);
                    breakdown = breakdown.push(
                        text(format!("  {category}: {mib:.1} MiB ({count} items)"))
                            .size(theme::meta(tokens)),
                    );
                }
            }
        } else {
            let mut search_index = 0u64;
            let mut ai_models = 0u64;
            let mut caches = 0u64;
            for (category, bytes, _) in &state.storage_rows {
                match category.as_str() {
                    "keyword_index" | "vector_index" => search_index += bytes,
                    "model_files" => ai_models += bytes,
                    "snippet_cache" | "search_cache" | "temporary_extraction" => caches += bytes,
                    _ => {}
                }
            }
            let mib = |b: u64| b as f64 / (1024.0 * 1024.0);
            for (label, bytes) in [
                (tr(locale, MessageKey::StorageGroupSearchIndex), search_index),
                (tr(locale, MessageKey::StorageGroupModels), ai_models),
                (tr(locale, MessageKey::StorageGroupCaches), caches),
            ] {
                if bytes > 0 {
                    breakdown = breakdown.push(
                        text(format!("  {label}: {:.1} MiB", mib(bytes))).size(theme::body(tokens)),
                    );
                }
            }
        }
    }

    // Safe cleanup uses secondary buttons; destructive uses danger.
    let content = column![
        breakdown,
        text(tr(locale, MessageKey::StorageSafeCleanupHeading)).size(theme::body(tokens)),
        row![
            components::secondary(
                tokens,
                tr(locale, MessageKey::StorageClearSnippets),
                Some(Message::CleanSnippets)
            ),
            components::secondary(
                tokens,
                tr(locale, MessageKey::StorageClearSearchCache),
                Some(Message::CleanSearchCache)
            ),
        ]
        .spacing(tokens.spacing.sm),
        text(tr(locale, MessageKey::StorageDangerHeading)).size(theme::body(tokens)),
        components::danger(
            tokens,
            tr(locale, MessageKey::StorageResetCatalog),
            Some(Message::AskResetCatalog)
        ),
        text(tr(locale, MessageKey::StorageResetWarning)).size(theme::meta(tokens)),
    ];
    page(tokens, content)
}

// ── Models view ──────────────────────────────────────────────────────────

pub fn models_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    let available = tr(locale, MessageKey::ModelsStatusAvailable);
    let missing = tr(locale, MessageKey::ModelsStatusMissing);
    let (embedding, reranker) = match state.capability {
        SearchCapability::KeywordOnly => (missing, missing),
        SearchCapability::Hybrid => (available, missing),
        SearchCapability::HybridWithRerank => (available, available),
    };
    let mut content = column![
        heading(tokens, tr(locale, MessageKey::ModelsTitle)),
        text(format!("{}: {embedding}", tr(locale, MessageKey::ModelsEmbeddingRole)))
            .size(theme::body(tokens)),
        text(format!("{}: {reranker}", tr(locale, MessageKey::ModelsRerankerRole)))
            .size(theme::body(tokens)),
    ];
    if state.capability == SearchCapability::KeywordOnly {
        content = content
            .push(text(tr(locale, MessageKey::ModelsKeywordOnlyHint)).size(theme::meta(tokens)));
    }
    page(tokens, content)
}

// ── Settings view ────────────────────────────────────────────────────────

pub fn settings_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;

    let mut language_row = row![].spacing(tokens.spacing.sm);
    for candidate in Locale::ALL {
        let label = text(candidate.display_name()).size(theme::body(tokens));
        let mut b = button(label).padding(Padding::from([tokens.spacing.sm, tokens.spacing.md]));
        if *candidate != locale {
            b = b.on_press(Message::SetLocale(*candidate));
        }
        language_row = language_row.push(b);
    }

    let mut theme_row = row![].spacing(tokens.spacing.sm);
    for candidate in Theme::ALL {
        let label = text(tr(locale, candidate.label_key())).size(theme::body(tokens));
        let mut b = button(label).padding(Padding::from([tokens.spacing.sm, tokens.spacing.md]));
        if *candidate != state.theme {
            b = b.on_press(Message::SetTheme(*candidate));
        }
        theme_row = theme_row.push(b);
    }

    let content = column![
        heading(tokens, tr(locale, MessageKey::SettingsTitle)),
        text(tr(locale, MessageKey::SettingsLanguageHeading)).size(theme::body(tokens)),
        language_row,
        text(tr(locale, MessageKey::SettingsThemeHeading)).size(theme::body(tokens)),
        theme_row,
        text(tr(locale, MessageKey::SettingsPrivacyHeading)).size(theme::body(tokens)),
        text(tr(locale, MessageKey::SettingsPrivacyLocalOnly)).size(theme::body(tokens)),
        text(tr(locale, MessageKey::SettingsAdvancedHeading)).size(theme::body(tokens)),
        row![
            button(
                text(if state.show_advanced {
                    tr(locale, MessageKey::SettingsAdvancedOn)
                } else {
                    tr(locale, MessageKey::SettingsAdvancedOff)
                })
                .size(theme::body(tokens)),
            )
            .on_press(Message::ToggleAdvanced),
            text(tr(locale, MessageKey::SettingsAdvancedHint)).size(theme::meta(tokens)),
        ]
        .spacing(tokens.spacing.sm),
    ];
    page(tokens, content)
}
