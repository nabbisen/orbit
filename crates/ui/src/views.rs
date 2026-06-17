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
use lucide_icons::iced as icons;
use orbok_models::SearchCapability;

/// Small icon+label button. `icon_fn` is a lucide `icon_*()` function.
/// Padding gives a comfortable click target (UX review: ~44px tall).
fn icon_btn<'a>(
    icon_el: iced::widget::Text<'a>,
    label: &'a str,
    msg: Message,
) -> iced::widget::Button<'a, Message> {
    button(row![icon_el.size(16), text(label).size(15)].spacing(6))
        .padding(Padding::from([12.0, 16.0]))
        .on_press(msg)
}

/// A friendly, actionable notice card (UX review §8). Shows a plain title,
/// an explanation, and — for problems — a recovery action. Confirmations
/// show a Dismiss action instead. Status is conveyed in words, never colour
/// alone, satisfying the accessibility requirement.
fn friendly_notice<'a>(
    locale: Locale,
    notice: &crate::notice::UserNotice,
) -> Element<'a, Message> {
    let action_label = notice
        .action(locale)
        .unwrap_or_else(|| tr(locale, MessageKey::NoticeDismiss));
    container(
        column![
            text(notice.title(locale).to_string()).size(18),
            text(notice.body(locale).to_string()).size(15),
            button(text(action_label.to_string()).size(15))
                .padding(Padding::from([10.0, 16.0]))
                .on_press(Message::ClearNotice),
        ]
        .spacing(8),
    )
    .padding(Padding::from([16.0, 16.0]))
    .width(Length::Fill)
    .into()
}

fn page<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content.spacing(10))
        .padding(Padding::from([28.0, 40.0]))
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
    let submit = icon_btn(icons::icon_search(), tr(locale, MessageKey::SearchButton), Message::SubmitSearch);

    let mut content = column![
        heading(tr(locale, MessageKey::NavSearch)),
        row![container(input).width(Length::Fill), submit].spacing(8),
    ];

    // Surface any active notice (problem or confirmation) at the top of the
    // page so failures are never silent (UX review P0).
    if let Some(notice) = &state.notice {
        content = content.push(friendly_notice(locale, notice));
    }

    // Search mode is "Auto" by default — only mature users need the
    // Exact/Conceptual switch. Hidden behind the Advanced toggle (less is more).
    if state.show_advanced {
        content = content.push(
            row![
                text(tr(locale, MessageKey::SearchModeLabel)).size(12),
                button(text(tr(locale, MessageKey::SearchModeAuto)).size(11))
                    .on_press(Message::SetSearchMode(orbok_search::SearchMode::Auto)),
                button(text(tr(locale, MessageKey::SearchModeExact)).size(11))
                    .on_press(Message::SetSearchMode(orbok_search::SearchMode::Exact)),
                button(text(tr(locale, MessageKey::SearchModeConceptual)).size(11))
                    .on_press(Message::SetSearchMode(orbok_search::SearchMode::Conceptual)),
            ]
            .spacing(4),
        );
    }

    if state.sources.is_empty() {
        // Required empty state: no sources (GUI design §7.6).
        content = content.push(
            column![
                text(tr(locale, MessageKey::SearchNoSourcesTitle)).size(18),
                text(tr(locale, MessageKey::SearchNoSourcesBody)).size(15),
                button(text(tr(locale, MessageKey::SearchAddSource)).size(15))
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
            content = content.push(text("Searching…").size(15));
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
                    let _is_selected = state.selected_result == Some(i); // TODO: visual highlight
                    let card = container(
                        column![
                            text(title_str.to_string()).size(15),
                            text(result.display_path.clone()).size(12),
                            if !heading_str.is_empty() { text(heading_str.to_string()).size(11) }
                            else { text("").size(11) },
                            text(snippet_str.chars().take(120).collect::<String>()).size(12),
                            {
                                // Less is more: by default show only status
                                // badges that affect trust (Stale/Missing).
                                // Match-type badges are advanced detail.
                                let shown: Vec<String> = if state.show_advanced {
                                    result.badges.clone()
                                } else {
                                    result.badges.iter()
                                        .filter(|b| {
                                            let l = b.to_lowercase();
                                            l.contains("stale") || l.contains("missing")
                                        })
                                        .cloned()
                                        .collect()
                                };
                                text(shown.join("  ")).size(11)
                            },
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

/// Sources view (§8): add-source input, list or empty state.
pub fn sources_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    // Add-source controls — folder picker button + optional manual path input.
    let add_btn = icon_btn(icons::icon_folder_plus(), tr(locale, MessageKey::SourcesAddFolder), Message::RequestAddSource);
    let add_input = text_input(
        "Or type a path manually…",
        &state.source_path_input,
    )
    .on_input(Message::SourcePathChanged)
    .on_submit(Message::RequestAddSource)
    .padding(8);
    let recursive_note = text("All sub-folders are scanned recursively.").size(12);
    let mut content = column![
        heading(tr(locale, MessageKey::SourcesTitle)),
        row![add_btn, container(add_input).width(Length::Fill)].spacing(8),
        recursive_note,
    ];

    if let Some(notice) = &state.notice {
        content = content.push(friendly_notice(locale, notice));
    }
    if state.sources.is_empty() {
        content = content.push(
            column![
                text(tr(locale, MessageKey::SourcesEmptyTitle)).size(18),
                text(tr(locale, MessageKey::SourcesEmptyBody)).size(15),
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
            let src_id = card.source_id.clone();
            content = content.push(
                container(
                    column![
                        text(card.display_name.clone()).size(15),
                        text(card.display_path.clone()).size(12),
                        text(source_summary(locale, card.indexed, card.stale, card.failed))
                            .size(12),
                        row![
                            text(status).size(11),
                            button(row![icons::icon_trash_2().size(12)].spacing(2))
                                .on_press(Message::SourceRemoved(src_id)),
                        ].spacing(8),
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
    // Less is more: always show "Indexed". Show queued/stale/failed cells
    // only when they are non-zero (or when advanced view is on), so a healthy
    // idle state is a single clean number rather than three zeros of noise.
    let mut cells = row![cell(tr(locale, MessageKey::IndexingHealthIndexed), h.indexed)]
        .spacing(10);
    if h.queued > 0 || state.show_advanced {
        cells = cells.push(cell(tr(locale, MessageKey::IndexingHealthQueued), h.queued));
    }
    if h.stale > 0 || state.show_advanced {
        cells = cells.push(cell(tr(locale, MessageKey::IndexingHealthStale), h.stale));
    }
    if h.failed > 0 || state.show_advanced {
        cells = cells.push(cell(tr(locale, MessageKey::IndexingHealthFailed), h.failed));
    }
    let content = column![
        heading(tr(locale, MessageKey::IndexingTitle)),
        cells,
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
        text(tr(locale, MessageKey::StorageIntro)).size(15),
        text(format!("{gib:.3} GiB total")).size(20),
    ]
    .spacing(4);

    if !state.storage_rows.is_empty() {
        if state.show_advanced {
            // Advanced: raw per-category breakdown.
            for (category, bytes, count) in &state.storage_rows {
                if *bytes > 0 || *count > 0 {
                    let mib = *bytes as f64 / (1024.0 * 1024.0);
                    breakdown = breakdown.push(
                        text(format!("  {category}: {mib:.1} MiB ({count} items)")).size(12),
                    );
                }
            }
        } else {
            // Default: three friendly buckets — no engine jargon (less is more).
            let mut search_index = 0u64;
            let mut ai_models = 0u64;
            let mut caches = 0u64;
            for (category, bytes, _) in &state.storage_rows {
                match category.as_str() {
                    "keyword_index" | "vector_index" => search_index += bytes,
                    "model_files" => ai_models += bytes,
                    "snippet_cache" | "search_cache" | "temporary_extraction" => caches += bytes,
                    _ => {} // persistent_catalog, logs — folded into total only
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
                        text(format!("  {label}: {:.1} MiB", mib(bytes))).size(15),
                    );
                }
            }
        }
    }

    let content = column![
        breakdown,
        text(tr(locale, MessageKey::StorageSafeCleanupHeading)).size(15),
        row![
            button(text(tr(locale, MessageKey::StorageClearSnippets)).size(15)),
            button(text(tr(locale, MessageKey::StorageClearSearchCache)).size(15)),
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
        text(tr(locale, MessageKey::SettingsAdvancedHeading)).size(15),
        row![
            button(
                text(if state.show_advanced {
                    tr(locale, MessageKey::SettingsAdvancedOn)
                } else {
                    tr(locale, MessageKey::SettingsAdvancedOff)
                })
                .size(13),
            )
            .on_press(Message::ToggleAdvanced),
            text(tr(locale, MessageKey::SettingsAdvancedHint)).size(11),
        ]
        .spacing(8),
    ];
    page(content)
}
