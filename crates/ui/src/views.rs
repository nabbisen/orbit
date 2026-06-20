//! Page view functions (GUI external design §7, §8–§12 wireframes).
//!
//! Each page is a plain function taking the view model and returning an
//! `Element` — the snora multi-view pattern. Empty states follow the
//! design's required empty-state set.
//!
//! Styling rule (RFC-032): no view here contains a literal font size,
//! padding, or spacing. Every such value is read from the active
//! `snora::design::Tokens` (`state.tokens`) through the [`crate::theme`]
//! helpers and the token spacing scale. snora is the sole gateway to the
//! design vocabulary.

pub mod wizard;
pub use wizard::wizard_view;

use crate::i18n::{Locale, MessageKey, files_indexed, search_result_count, source_summary, tr};
use crate::state::{AppState, Message};
use crate::theme::{self, Theme};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Padding};
use orbok_models::SearchCapability;
use snora::design::Tokens;
use snora::lucide;

/// Render a lucide icon as a sized Text widget using char::from().
/// This is the same technique snora uses in icon_element_sized() and
/// avoids the iced type-parameter mismatch that lucide_icons::iced::icon_*()
/// can cause when multiple iced_core versions are in the dep graph.
///
/// `size` is an icon glyph dimension, not body typography, so it stays an
/// explicit argument rather than a typography token.
fn icon_text<'a>(glyph: char, size: f32) -> iced::widget::Text<'a> {
    iced::widget::text(glyph.to_string())
        .font(iced::Font::with_name("lucide"))
        .size(size)
}

/// Small icon+label button. Token padding gives a comfortable click target
/// (≥44px at the default density — see RFC-034 target-size rule).
fn icon_btn<'a>(
    tokens: &Tokens,
    icon_el: iced::widget::Text<'a>,
    label: &'a str,
    msg: Message,
) -> iced::widget::Button<'a, Message> {
    button(row![icon_el, text(label).size(theme::body(tokens))].spacing(tokens.spacing.sm))
        .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
        .on_press(msg)
}

/// A friendly, actionable notice card (UX review §8). Shows a plain title,
/// an explanation, and — for problems — a recovery action. Confirmations
/// show a Dismiss action instead. Status is conveyed in words, never colour
/// alone, satisfying the accessibility requirement.
fn friendly_notice<'a>(
    tokens: &'a Tokens,
    locale: Locale,
    notice: &crate::notice::UserNotice,
) -> Element<'a, Message> {
    use snora::design::notice::Notice;

    // Render via the Snora Design notice primitive: tone-driven, WCAG-AA
    // verified colors, keyboard-reachable action/dismiss controls. The
    // UserNotice domain type still owns the semantics and i18n; the snora
    // primitive owns the accessible presentation.
    let mut builder = Notice::new(tokens, notice.tone(), notice.body(locale).to_string())
        .title(notice.title(locale).to_string());

    // Problem notices get a recovery action that clears the notice;
    // confirmations get a dismiss control instead.
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

/// Search view (§7): input, capability notice, empty states.
pub fn search_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    let input = text_input(tr(locale, MessageKey::SearchPlaceholder), &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::SubmitSearch)
        .padding(tokens.spacing.sm);
    let submit = icon_btn(
        tokens,
        icon_text(char::from(lucide::Search), 13.0),
        tr(locale, MessageKey::SearchButton),
        Message::SubmitSearch,
    );

    let mut content = column![
        heading(tokens, tr(locale, MessageKey::NavSearch)),
        row![container(input).width(Length::Fill), submit].spacing(tokens.spacing.sm),
    ];

    // Surface any active notice (problem or confirmation) at the top of the
    // page so failures are never silent (UX review P0).
    if let Some(notice) = &state.notice {
        content = content.push(friendly_notice(tokens, locale, notice));
    }

    // Search mode is "Auto" by default — only mature users need the
    // Exact/Conceptual switch. Hidden behind the Advanced toggle (less is more).
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
        // Required empty state: no sources (GUI design §7.6).
        content = content.push(
            column![
                text(tr(locale, MessageKey::SearchNoSourcesTitle)).size(theme::title(tokens)),
                text(tr(locale, MessageKey::SearchNoSourcesBody)).size(theme::body(tokens)),
                button(text(tr(locale, MessageKey::SearchAddSource)).size(theme::body(tokens)))
                    .on_press(Message::Switch(crate::state::ViewId::Sources)),
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
                        std::borrow::Cow::Owned(format!("▶  {title_raw}"))
                    } else {
                        std::borrow::Cow::Borrowed(title_raw)
                    };
                    let title_str: &str = &title_str;
                    let snippet_str = result.snippet.as_deref().unwrap_or("(source unavailable)");
                    let heading_str = result.heading_path.as_deref().unwrap_or("");
                    let card = container(
                        column![
                            text(title_str.to_string()).size(theme::body(tokens)),
                            text(result.display_path.clone()).size(theme::meta(tokens)),
                            if !heading_str.is_empty() {
                                text(heading_str.to_string()).size(theme::meta(tokens))
                            } else {
                                text("").size(theme::meta(tokens))
                            },
                            text(snippet_str.chars().take(120).collect::<String>())
                                .size(theme::meta(tokens)),
                            {
                                // Less is more: by default show only status
                                // badges that affect trust (Stale/Missing).
                                // Match-type badges are advanced detail.
                                let shown: Vec<String> = if state.show_advanced {
                                    result.badges.clone()
                                } else {
                                    result
                                        .badges
                                        .iter()
                                        .filter(|b| {
                                            let l = b.to_lowercase();
                                            l.contains("stale") || l.contains("missing")
                                        })
                                        .cloned()
                                        .collect()
                                };
                                text(shown.join("  ")).size(theme::meta(tokens))
                            },
                        ]
                        .spacing(tokens.spacing.xs),
                    )
                    .padding(tokens.spacing.md);
                    content = content.push(button(card).on_press(Message::SelectResult(i)));
                }
            }
        }
    }
    page(tokens, content)
}

/// Sources view (§8): add-source input, list or empty state.
pub fn sources_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    // Add-source controls — folder picker button + optional manual path input.
    let add_btn = icon_btn(
        tokens,
        icon_text(char::from(lucide::FolderPlus), 13.0),
        tr(locale, MessageKey::SourcesAddFolder),
        Message::RequestAddSource,
    );
    let add_input = text_input("Or type a path manually…", &state.source_path_input)
        .on_input(Message::SourcePathChanged)
        .on_submit(Message::RequestAddSource)
        .padding(tokens.spacing.sm);
    let recursive_note =
        text("All sub-folders are scanned recursively.").size(theme::meta(tokens));
    let mut content = column![
        heading(tokens, tr(locale, MessageKey::SourcesTitle)),
        row![add_btn, container(add_input).width(Length::Fill)].spacing(tokens.spacing.sm),
        recursive_note,
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
            let status = if card.active {
                tr(locale, MessageKey::SourcesStatusActive)
            } else {
                tr(locale, MessageKey::SourcesStatusPaused)
            };
            let src_id = card.source_id.clone();
            content = content.push(
                container(
                    column![
                        text(card.display_name.clone()).size(theme::body(tokens)),
                        text(card.display_path.clone()).size(theme::meta(tokens)),
                        text(source_summary(locale, card.indexed, card.stale, card.failed))
                            .size(theme::meta(tokens)),
                        row![
                            text(status).size(theme::meta(tokens)),
                            button(row![icon_text(char::from(lucide::Trash2), 12.0)].spacing(0))
                                .on_press(Message::SourceRemoved(src_id)),
                        ]
                        .spacing(tokens.spacing.sm),
                    ]
                    .spacing(tokens.spacing.xs),
                )
                .padding(tokens.spacing.md),
            );
        }
    }
    page(tokens, content)
}

/// Indexing view (§9): health summary cards.
pub fn indexing_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    let h = state.health;
    let cell = |label: &'static str, value: u64| {
        container(
            column![
                text(label).size(theme::meta(tokens)),
                text(value.to_string()).size(theme::title(tokens))
            ]
            .spacing(tokens.spacing.xs),
        )
        .padding(tokens.spacing.md)
    };
    // Less is more: always show "Indexed". Show queued/stale/failed cells
    // only when they are non-zero (or when advanced view is on), so a healthy
    // idle state is a single clean number rather than three zeros of noise.
    let mut cells = row![cell(
        tr(locale, MessageKey::IndexingHealthIndexed),
        h.indexed
    )]
    .spacing(tokens.spacing.md);
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
        heading(tokens, tr(locale, MessageKey::IndexingTitle)),
        cells,
        text(if h.queued == 0 {
            tr(locale, MessageKey::IndexingIdle).to_string()
        } else {
            files_indexed(locale, h.indexed)
        })
        .size(theme::body(tokens)),
    ];
    page(tokens, content)
}

/// Storage view (§10): safe cleanup vs danger zone, with real data.
pub fn storage_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;

    // If confirmation is pending for a destructive reset, show the dialog only.
    if state.confirm_reset {
        let content = column![
            text(tr(locale, MessageKey::StorageResetCatalog)).size(theme::title(tokens)),
            text(tr(locale, MessageKey::StorageResetWarning)).size(theme::body(tokens)),
            row![
                button(text(tr(locale, MessageKey::Cancel)).size(theme::body(tokens)))
                    .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
                    .on_press(Message::CancelResetCatalog),
                button(
                    text(tr(locale, MessageKey::StorageResetCatalog)).size(theme::body(tokens))
                )
                .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
                .on_press(Message::ConfirmResetCatalog),
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
            // Advanced: raw per-category breakdown.
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
                (
                    tr(locale, MessageKey::StorageGroupSearchIndex),
                    search_index,
                ),
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

    let content = column![
        breakdown,
        text(tr(locale, MessageKey::StorageSafeCleanupHeading)).size(theme::body(tokens)),
        row![
            button(text(tr(locale, MessageKey::StorageClearSnippets)).size(theme::body(tokens)))
                .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
                .on_press(Message::CleanSnippets),
            button(
                text(tr(locale, MessageKey::StorageClearSearchCache)).size(theme::body(tokens))
            )
            .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
            .on_press(Message::CleanSearchCache),
        ]
        .spacing(tokens.spacing.sm),
        text(tr(locale, MessageKey::StorageDangerHeading)).size(theme::body(tokens)),
        button(text(tr(locale, MessageKey::StorageResetCatalog)).size(theme::body(tokens)))
            .padding(Padding::from([tokens.spacing.md, tokens.spacing.lg]))
            .on_press(Message::AskResetCatalog),
        text(tr(locale, MessageKey::StorageResetWarning)).size(theme::meta(tokens)),
    ];
    page(tokens, content)
}

/// Models view (§11): role statuses and keyword-only hint.
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
        text(format!(
            "{}: {embedding}",
            tr(locale, MessageKey::ModelsEmbeddingRole)
        ))
        .size(theme::body(tokens)),
        text(format!(
            "{}: {reranker}",
            tr(locale, MessageKey::ModelsRerankerRole)
        ))
        .size(theme::body(tokens)),
    ];
    if state.capability == SearchCapability::KeywordOnly {
        content =
            content.push(text(tr(locale, MessageKey::ModelsKeywordOnlyHint)).size(theme::meta(tokens)));
    }
    page(tokens, content)
}

/// Settings view (§12): language picker, privacy, advanced, and theme.
pub fn settings_view(state: &AppState) -> Element<'_, Message> {
    let locale = state.locale;
    let tokens = &state.tokens;
    let mut language_row = row![].spacing(tokens.spacing.sm);
    for candidate in Locale::ALL {
        let label = text(candidate.display_name()).size(theme::body(tokens));
        let mut b = button(label);
        if *candidate != locale {
            b = b.on_press(Message::SetLocale(*candidate));
        }
        language_row = language_row.push(b);
    }

    // Theme picker (RFC-032): the active theme is rendered without an
    // on_press, exactly like the active locale, so it reads as "selected".
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
