//! RFC-033 component adapter tests: tone mapping, badge invariant, smoke builds.

use crate::components::{badge_tone, status_badge, tone_icon};
use crate::state::Message;
use snora::design::{Tokens, Tone};

// RFC-033 §5.2: the badge_tone mapping is stable — shared by UI and RFC-035 CVD fixture.
#[test]
fn badge_tone_mapping() {
    let cases: &[(&str, Tone)] = &[
        ("missing source", Tone::Danger),
        ("Missing", Tone::Danger),
        ("stale", Tone::Warning),
        ("Stale index", Tone::Warning),
        ("semantic", Tone::Accent),
        ("Reranked", Tone::Accent),
        ("keyword", Tone::Info),
        ("Keyword match", Tone::Info),
        ("current", Tone::Success),
        ("Current", Tone::Success),
        ("temporary", Tone::Neutral),
        ("unknown badge", Tone::Neutral),
        ("", Tone::Neutral),
    ];
    for (label, expected) in cases {
        assert_eq!(badge_tone(label), *expected, "badge_tone({label:?})");
    }
}

// RFC-033 + RFC-034 §5.2: each tone maps to a non-null icon glyph; badges build
// without panicking for representative labels (text + icon invariant).
#[test]
fn status_badge_label_and_icon_invariant() {
    for tone in [
        Tone::Success,
        Tone::Warning,
        Tone::Danger,
        Tone::Info,
        Tone::Accent,
        Tone::Neutral,
    ] {
        assert_ne!(
            tone_icon(tone) as u32,
            0,
            "tone_icon for {tone:?} must be non-null"
        );
    }
    let tokens = Tokens::light();
    for label in [
        "stale",
        "missing",
        "keyword",
        "semantic",
        "current",
        "temporary",
    ] {
        let _ = status_badge(&tokens, label, badge_tone(label));
    }
}

// RFC-033 §8: adapters build Elements for normal and edge cases.
#[test]
fn component_smoke_result_card() {
    let tokens = Tokens::light();
    // Normal unselected card.
    let _ = crate::components::result_card(
        &tokens,
        "My document.md".to_string(),
        "/home/user/My document.md".to_string(),
        "Section heading".to_string(),
        "A short snippet of content…".to_string(),
        &["stale".to_string(), "keyword".to_string()],
        false,
        false,
        Message::SelectResult(0),
    );
    // Selected card with no heading and empty badges.
    let _ = crate::components::result_card(
        &tokens,
        "▶  selected.pdf".to_string(),
        "/docs/selected.pdf".to_string(),
        String::new(),
        "(source unavailable)".to_string(),
        &[],
        false,
        true,
        Message::SelectResult(1),
    );
}

#[test]
fn component_smoke_source_card() {
    let tokens = Tokens::light();
    let _ = crate::components::source_card(
        &tokens,
        "Documents".to_string(),
        "/home/user/Documents".to_string(),
        "812 indexed · 0 stale".to_string(),
        "Active",
        Message::SourceRemoved("src-1".to_string()),
    );
}

#[test]
fn component_smoke_health_cell() {
    let tokens = Tokens::light();
    let _ = crate::components::health_cell(&tokens, "Indexed", 812);
    let _ = crate::components::health_cell(&tokens, "Failed", 0);
}

#[test]
fn component_smoke_action_buttons() {
    let tokens = Tokens::light();
    let _ = crate::components::primary(&tokens, "Save", Some(Message::ToggleAdvanced));
    let _ = crate::components::primary(&tokens, "Save", None);
    let _ = crate::components::secondary(&tokens, "Cancel", Some(Message::ClearNotice));
    let _ = crate::components::ghost(&tokens, "Details", None);
    let _ = crate::components::danger(&tokens, "Delete", Some(Message::AskResetCatalog));
    let _ = crate::components::danger(&tokens, "Delete", None);
}

#[test]
fn component_smoke_progress() {
    let tokens = Tokens::light();
    let _ = crate::components::job_progress(&tokens, "Indexing...", Some(0.42));
    let _ = crate::components::job_progress(&tokens, "Queued", None);
}
