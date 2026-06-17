//! Page view functions (GUI external design §7, §8–§12 wireframes).
//!
//! Each page is a plain function taking the view model and returning an
//! `Element` — the snora multi-view pattern. Empty states follow the
//! design's required empty-state set.

pub mod wizard;
pub use wizard::wizard_view;

use crate::i18n::{Locale, MessageKey, files_indexed, search_result_count, source_summary, tr};
use crate::state::{AppState, Message};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Padding};
use orbok_models::SearchCapability;

fn page<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content.spacing(10))
        .padding(Padding::from([24.0, 32.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn heading(label: &str) -> iced::widget::Text<'_> {
    text(label.to_string()).size(26)
}

/// Search view (§7): input, capability notice, empty states.
pub fn search_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let input = text_input(tr(locale, MessageKey::SearchPlaceholder), &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::SubmitSearch)
        .padding(8);
    let submit = button(text(tr(locale, MessageKey::SearchButton)).size(13))
        .on_press(Message::SubmitSearch);

    let mode = state.search_mode;
    let mode_selector = row![
        text(tr(locale, MessageKey::SearchModeLabel)).size(12),
        button(text(tr(locale, MessageKey::SearchModeAuto)).size(11))
            .on_press(Message::SetSearchMode(orbok_search::SearchMode::Auto)),
        button(text(tr(locale, MessageKey::SearchModeExact)).size(11))
            .on_press(Message::SetSearchMode(orbok_search::SearchMode::Exact)),
        button(text(tr(locale, MessageKey::SearchModeConceptual)).size(11))
            .on_press(Message::SetSearchMode(orbok_search::SearchMode::Conceptual)),
    ]
    .spacing(4);
    let mut content = column![
        heading(tr(locale, MessageKey::NavSearch)),
        row![container(input).width(Length::Fill), submit].spacing(8),
        mode_selector,
    ];

    if state.sources.is_empty() {
        // Required empty state: no sources (GUI design §7.6).
        content = content.push(
            column![
                text(tr(locale, MessageKey::SearchNoSourcesTitle)).size(18),
                text(tr(locale, MessageKey::SearchNoSourcesBody)).size(13),
                button(text(tr(locale, MessageKey::SearchAddSource)).size(13))
                    .on_press(Message::Switch(crate::state::ViewId::Sources)),
            ]
            .spacing(6),
        );
    } else {
        if state.capability == SearchCapability::KeywordOnly {
            content = content
                .push(text(tr(locale, MessageKey::SearchKeywordOnlyNotice)).size(12));
        }
        if state.search_running {
            content = content.push(text("Searching…").size(13));
        } else if let Some(last) = &state.last_query {
            if state.search_results.is_empty() {
                content = content.push(
                    column![
                        text(tr(locale, MessageKey::SearchNoResults)).size(15),
                        text(format!("Query: {last}")).size(12),
                    ]
                    .spacing(4),
                );
            } else {
                content = content.push(
                    text(search_result_count(locale, state.search_results.len())).size(12),
                );
                for (i, result) in state.search_results.iter().enumerate() {
                    let title_str = result.title.as_deref().unwrap_or(&result.display_path);
                    let snippet_str = result
                        .snippet
                        .as_deref()
                        .unwrap_or("(source unavailable)");
                    let heading_str = result.heading_path.as_deref().unwrap_or("");
                    let is_selected = state.selected_result == Some(i);
                    let card = container(
                        column![
                            text(title_str.to_string()).size(15),
                            text(result.display_path.clone()).size(11),
                            if !heading_str.is_empty() { text(heading_str.to_string()).size(11) }
                            else { text("").size(11) },
                            text(snippet_str.chars().take(120).collect::<String>()).size(12),
                            text(result.badges.join("  ")).size(11),
                        ]
                        .spacing(2),
                    )
                    .padding(10);
                    content = content.push(
                        button(card).on_press(Message::SelectResult(i)),
                    );
                }
            }
        }
    }
    page(content)
}

/// Sources view (§8): list or empty state.
pub fn sources_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let mut content = column![heading(tr(locale, MessageKey::SourcesTitle))];
    if state.sources.is_empty() {
        content = content.push(
            column![
                text(tr(locale, MessageKey::SourcesEmptyTitle)).size(18),
                text(tr(locale, MessageKey::SourcesEmptyBody)).size(13),
                button(text(tr(locale, MessageKey::SourcesAddFolder)).size(13)),
            ]
            .spacing(6),
        );
    } else {
        for card in &state.sources {
            let status = if card.active {
                tr(locale, MessageKey::SourcesStatusActive)
            } else {
                tr(locale, MessageKey::SourcesStatusPaused)
            };
            content = content.push(
                container(
                    column![
                        text(card.display_name.clone()).size(15),
                        text(card.display_path.clone()).size(11),
                        text(source_summary(locale, card.indexed, card.stale, card.failed))
                            .size(12),
                        text(status).size(11),
                    ]
                    .spacing(2),
                )
                .padding(10),
            );
        }
    }
    page(content)
}

/// Indexing view (§9): health summary cards.
pub fn indexing_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let h = state.health;
    let cell = |label: &'static str, value: u64| {
        container(column![text(label).size(12), text(value.to_string()).size(20)].spacing(2))
            .padding(10)
    };
    let content = column![
        heading(tr(locale, MessageKey::IndexingTitle)),
        row![
            cell(tr(locale, MessageKey::IndexingHealthIndexed), h.indexed),
            cell(tr(locale, MessageKey::IndexingHealthQueued), h.queued),
            cell(tr(locale, MessageKey::IndexingHealthStale), h.stale),
            cell(tr(locale, MessageKey::IndexingHealthFailed), h.failed),
        ]
        .spacing(10),
        text(if h.queued == 0 {
            tr(locale, MessageKey::IndexingIdle).to_string()
        } else {
            files_indexed(locale, h.indexed)
        })
        .size(13),
    ];
    page(content)
}

/// Storage view (§10): safe cleanup vs danger zone, with real data.
pub fn storage_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let total_bytes: u64 = state.storage_rows.iter().map(|(_, b, _)| b).sum();
    let gib = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let mut breakdown = column![
        text(tr(locale, MessageKey::StorageTitle)).size(26),
        text(tr(locale, MessageKey::StorageIntro)).size(13),
        text(format!("{gib:.3} GiB total")).size(20),
    ]
    .spacing(4);

    if !state.storage_rows.is_empty() {
        for (category, bytes, count) in &state.storage_rows {
            if *bytes > 0 || *count > 0 {
                let mib = *bytes as f64 / (1024.0 * 1024.0);
                breakdown = breakdown.push(
                    text(format!("  {category}: {mib:.1} MiB ({count} items)")).size(12),
                );
            }
        }
    }

    let content = column![
        breakdown,
        text(tr(locale, MessageKey::StorageSafeCleanupHeading)).size(15),
        row![
            button(text(tr(locale, MessageKey::StorageClearSnippets)).size(13)),
            button(text(tr(locale, MessageKey::StorageClearSearchCache)).size(13)),
        ]
        .spacing(8),
        text(tr(locale, MessageKey::StorageDangerHeading)).size(15),
        button(text(tr(locale, MessageKey::StorageResetCatalog)).size(13)),
        text(tr(locale, MessageKey::StorageResetWarning)).size(11),
    ];
    page(content)
}

/// Models view (§11): role statuses and keyword-only hint.
pub fn models_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let available = tr(locale, MessageKey::ModelsStatusAvailable);
    let missing = tr(locale, MessageKey::ModelsStatusMissing);
    let (embedding, reranker) = match state.capability {
        SearchCapability::KeywordOnly => (missing, missing),
        SearchCapability::Hybrid => (available, missing),
        SearchCapability::HybridWithRerank => (available, available),
    };
    let mut content = column![
        heading(tr(locale, MessageKey::ModelsTitle)),
        text(format!("{}: {embedding}", tr(locale, MessageKey::ModelsEmbeddingRole))).size(14),
        text(format!("{}: {reranker}", tr(locale, MessageKey::ModelsRerankerRole))).size(14),
    ];
    if state.capability == SearchCapability::KeywordOnly {
        content = content.push(text(tr(locale, MessageKey::ModelsKeywordOnlyHint)).size(12));
    }
    page(content)
}

/// Settings view (§12): language picker + privacy section.
pub fn settings_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let mut language_row = row![].spacing(8);
    for candidate in Locale::ALL {
        let label = text(candidate.display_name()).size(13);
        let mut b = button(label);
        if *candidate != locale {
            b = b.on_press(Message::SetLocale(*candidate));
        }
        language_row = language_row.push(b);
    }
    let content = column![
        heading(tr(locale, MessageKey::SettingsTitle)),
        text(tr(locale, MessageKey::SettingsLanguageHeading)).size(15),
        language_row,
        text(tr(locale, MessageKey::SettingsPrivacyHeading)).size(15),
        text(tr(locale, MessageKey::SettingsPrivacyLocalOnly)).size(13),
    ];
    page(content)
}
