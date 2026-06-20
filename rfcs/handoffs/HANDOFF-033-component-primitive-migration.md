# HANDOFF-033 — Component Primitive Migration

**RFC:** `rfcs/proposed/033-component-primitive-migration.md`
**Owner crate(s):** `orbok-ui`
**Prereqs:** HANDOFF-032 merged (tokens threaded). **Blocks:** 034.
**Release:** current version — do **not** bump.

---

## 0. Orientation

Stand up `orbok-ui/src/components.rs` as the single adapter from orbok
view-models to Snora Design primitives, then migrate each view to call it.
Net effect: cards/buttons/badges/progress are built once, consistently,
token-driven, with destructive actions visually distinct and disabled actions
truly disabled. `views.rs` shrinks because the per-card composition moves out.

Verified snora 0.25 surface (do not invent beyond this):
- `snora::design::button::{primary, secondary, ghost, danger}` and each
  `*_maybe(tokens, label, Option<Message>)` for disabled state.
- `snora::design::card::{surface, raised, selected}(tokens, content)`.
- `snora::design::chip::{filter, removable}(tokens, label, selected, on_toggle[, on_remove])`.
- `snora::design::progress::{row, card}(tokens, label, Option<f32>, Tone)`.
- `snora::design::notice::Notice` builder (already used).
- `snora::design::variants::Tone::{Neutral, Accent, Success, Warning, Danger, Info}`.

---

## 1. Files

| File | Action |
|---|---|
| `crates/ui/src/components.rs` | **new** — adapters + Tone mapping + inventory doc |
| `crates/ui/src/lib.rs` | `pub mod components;` |
| `crates/ui/src/views.rs` | replace inline cards/buttons/badges with `components::*` calls |
| `crates/ui/src/views/wizard.rs` | wizard buttons → `button::*` via components |
| `crates/ui/src/notice.rs` | unchanged (already primitive); referenced by inventory |
| `crates/ui/src/tests.rs` (+ `tests/smoke_views.rs`) | add component smoke + Tone-map tests |
| `docs/src/maintainers/architecture.md` | add the primitive inventory + gateway rule |

Keep both `views.rs` and `components.rs` < 500 ELOC (strong-split threshold). If
`components.rs` approaches it, split into `components/cards.rs` +
`components/controls.rs` (2018 module style, no `mod.rs` needed).

---

## 2. `components.rs` skeleton

```rust
//! orbok view-model → Snora Design primitive adapters (RFC-033).
//!
//! Views call these; they never call `snora::design::{button,card,…}`
//! directly, so a future primitive swap touches only this file. snora is the
//! sole gateway for UI primitives (cf. the lucide-icons gateway rule).

use crate::state::{Message, SourceCard /* etc. */};
use iced::Element;
use snora::design::{Tokens, button, card, progress, variants::Tone};

// ── Status badges (text + tone; never colour alone — RFC-034 §5.2) ────
pub fn status_badge<'a>(tokens: &Tokens, label: &str, tone: Tone) -> Element<'a, Message> { /* … */ }

/// Map an orbok badge string to a semantic tone. Stable, table-driven.
pub fn badge_tone(label: &str) -> Tone {
    let l = label.to_lowercase();
    if l.contains("missing") { Tone::Danger }
    else if l.contains("stale") { Tone::Warning }
    else if l.contains("semantic") || l.contains("rerank") { Tone::Accent }
    else if l.contains("keyword") { Tone::Info }
    else if l.contains("current") { Tone::Success }
    else { Tone::Neutral }
}

// ── Cards ─────────────────────────────────────────────────────────────
pub fn result_card<'a>(tokens: &Tokens, vm: &ResultCardVm, selected: bool, on_select: Message) -> Element<'a, Message>;
pub fn source_card<'a>(tokens: &Tokens, c: &SourceCard, on_remove: Message) -> Element<'a, Message>;
pub fn model_card<'a>(tokens: &Tokens, /* model vm */ ) -> Element<'a, Message>;
pub fn cleanup_action_card<'a>(tokens: &Tokens, title: &str, body: &str, action: CleanupAction) -> Element<'a, Message>;

// ── Action buttons (thin pass-throughs that fix label sizing/tone) ────
pub fn primary<'a>(tokens: &Tokens, label: &str, on: Option<Message>) -> Element<'a, Message>
    { button::primary_maybe(tokens, label, on).into() }
pub fn secondary<'a>(tokens: &Tokens, label: &str, on: Option<Message>) -> Element<'a, Message>
    { button::secondary_maybe(tokens, label, on).into() }
pub fn ghost<'a>(tokens: &Tokens, label: &str, on: Option<Message>) -> Element<'a, Message>
    { button::ghost_maybe(tokens, label, on).into() }
pub fn danger<'a>(tokens: &Tokens, label: &str, on: Option<Message>) -> Element<'a, Message>
    { button::danger_maybe(tokens, label, on).into() }

// ── Progress ──────────────────────────────────────────────────────────
pub fn job_progress<'a>(tokens: &'a Tokens, label: &'a str, value: Option<f32>) -> Element<'a, Message>
    { progress::row(tokens, label, value, Tone::Accent) }
```

For `result_card`, retain existing behavior: the `▶` selected marker can stay as
a textual cue *and* the card uses `card::selected` when `selected`. Wrap the card
in a ghost-styled `button(...).on_press(on_select)` since `card::selected` is
non-interactive in 0.25 (documented).

For icon+label buttons (e.g. search submit, add folder, remove source), keep the
lucide icon via the existing `icon_text` helper and compose
`row![icon, text(label)]` as the button content, but route the **button styling**
through `button::*`. (snora's button builders take a label `String`; for icon
buttons either (a) use a raw `iced::widget::button` with
`.style(move |_t,s| snora::design::style::button::primary(&t, s))` so styling is
still snora, or (b) request an upstream icon-button overload. Use (a) now; note
(b) as an upstream candidate.)

---

## 3. Per-view migration checklist

**Storage view (do first — safety win):**
- "Clear snippets" / "Clear search cache" → `components::secondary`.
- "Remove replaced stale indexes" → `components::secondary`.
- "Reset catalog…" and the confirm dialog's confirm button → `components::danger`.
- "Cancel" in the confirm dialog → `components::ghost`.
- Storage breakdown rows → `card::surface` wrapper; numbers stay via i18n
  (RFC-035 finalizes formatting).

**Search view:**
- Result card composition (the big inline `container(column![…])`) → replace with
  `components::result_card`.
- Badge string `text(shown.join("  "))` → a `row!` of `components::status_badge`
  for each label with `badge_tone(label)`. Keep the "less is more" filter
  (Stale/Missing only unless `show_advanced`).
- Submit button: disabled while `search_running` →
  `components::primary(tokens, label, (!running).then_some(Message::SubmitSearch))`.

**Sources view:**
- Source card → `components::source_card`; the Trash2 remove action →
  `components::danger` (DangerZone) with existing confirm pattern.

**Models view:**
- Model rows → `components::model_card`; status → `status_badge`
  (Available→Success, Missing→Danger/Warning, Optional→Neutral).
- Install → `components::primary`; Locate/Validate → `components::secondary`.

**Indexing view:**
- Running job → `components::job_progress(tokens, label, Some(fraction))`;
  queued/unknown → `None` (indeterminate).

**Wizard:**
- Back → `components::ghost`; Continue/Accept → `components::primary`;
  Skip → `components::secondary`.

---

## 4. Inventory doc (authoritative copy lives in architecture.md)

Reproduce RFC-033 §5.2 table in `docs/src/maintainers/architecture.md` under a
"UI component inventory" heading, with the two **bespoke** rows
(confirmation dialog, two-pane split, wizard stepper) flagged and each carrying
its one-line rationale + upstream-candidate note. File two snora issues
(modal/dialog primitive; split-pane primitive) referencing RFC-033; link them in
the doc.

---

## 5. Tests

In `crates/ui/src/tests.rs` (+ `tests/smoke_views.rs`):

1. `badge_tone_mapping` — table: ("missing source"→Danger), ("stale"→Warning),
   ("semantic"→Accent), ("reranked"→Accent), ("keyword"→Info),
   ("current"→Success), ("temporary"→Neutral), unknown→Neutral.
2. `status_badge_has_label` — building a badge with empty label is rejected or
   yields a non-empty rendered label (invariant for RFC-034).
3. `component_smoke` — each adapter builds an `Element` for a normal + edge case
   (empty title/path, indeterminate progress, `None` press handler).
4. `disabled_action_is_disabled` — `primary(tokens, l, None)` yields a button
   without an active press handler (use `iced_test` if it exposes this; else a
   builder-level assertion).
5. Existing `smoke_views` still pass after each view is migrated.

---

## 6. Definition of done

- [ ] `components.rs` adapters cover every §5.2 inventory row.
- [ ] Storage/Search/Sources/Models/Indexing/Wizard migrated to `components::*`.
- [ ] Reset/Delete/Remove render with `button::danger`; submit-while-running
      and model-dependent actions are truly disabled (`*_maybe(None)`).
- [ ] No view module builds a styled button/card/badge from raw iced widgets for
      a role with a snora primitive (grep gate extended to flag this).
- [ ] Inventory table in architecture.md; two upstream snora issues filed/linked.
- [ ] `views.rs` and `components.rs` each < 500 ELOC.
- [ ] Tests pass; `cargo build --workspace`/`cargo test --workspace`
      warning-free and green.
- [ ] CHANGELOG entry under the current version (no version bump).
