# HANDOFF-035 — Inclusive Design

**RFC:** `rfcs/done/035-inclusive-design.md`
**Owner crate(s):** `orbok-ui` (preferences + Settings surface + formatting),
`orbok-app` (OS probes + persistence)
**Prereqs:** HANDOFF-032 (tokens/themes). Overlaps 033/034 on Settings + badges.
**Release:** current version — do **not** bump.

---

## 0. Orientation

Add the small, persisted inclusivity preference set on top of the token
foundation: **text scale**, **reduced motion**, the full **theme picker** in a
plain-language Settings surface, a **color-vision-safe** status guarantee (icon
bound to each tone), and **locale-aware formatting** routed through i18n. Confirm
**RTL readiness** by audit. Nothing here re-implements locale data or adds a
color editor.

---

## 1. Files

| File | Action |
|---|---|
| `crates/ui/src/state.rs` | add `text_scale: TextScale`, `reduced_motion: bool`; messages `SetTextScale`, `SetReducedMotion`, persist variants |
| `crates/ui/src/theme.rs` | typography helpers multiply by `text_scale.factor()` |
| `crates/ui/src/views.rs` | Settings → Appearance + Accessibility sections (plain language) |
| `crates/ui/src/components.rs` | bind a lucide icon to each status tone (CVD-safe badge) |
| `crates/ui/src/i18n.rs` (+ en/ja) | locale-aware number/byte/date fns; route view `format!` sites through them |
| `crates/app/src/settings.rs` | add `text_scale`, `reduced_motion` fields |
| `crates/app/src/bootstrap.rs` | load prefs; resolve OS reduce-motion default |
| `crates/app/src/main.rs` | handle persist messages |
| `crates/ui/src/tests.rs` | scale, persistence, reduced-motion default, CVD fixture, formatting, direction audit |
| `docs/src/users/settings.md` | document Appearance/Accessibility |
| `docs/src/maintainers/accessibility.md` | add CVD + RTL-readiness notes (shared with 034) |

---

## 2. Text scale

```rust
// state.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextScale { #[default] Default, Large, Larger }
impl TextScale {
    pub fn factor(self) -> f32 { match self { Self::Default=>1.0, Self::Large=>1.15, Self::Larger=>1.3 } }
    pub fn as_str(self) -> &'static str { match self { Self::Default=>"default", Self::Large=>"large", Self::Larger=>"larger" } }
    pub fn parse(s:&str)->Option<Self>{ Some(match s {"default"=>Self::Default,"large"=>Self::Large,"larger"=>Self::Larger,_=>return None}) }
    pub const ALL: &'static [TextScale] = &[Self::Default, Self::Large, Self::Larger];
}
```

Apply centrally in `theme.rs` — the helpers gain the scale (read from state):

```rust
// theme.rs — scale-aware variants used by views
pub fn body_scaled(t:&Tokens, s:TextScale) -> Pixels { Pixels(body(t).0 * s.factor()) }
// …heading_scaled, title_scaled, meta_scaled, label_scaled similarly.
```

Views call the `*_scaled(tokens, state.text_scale)` forms. (Alternatively, fold
the factor into a single `Sizes { … }` struct built once per view from
`(tokens, text_scale)` to keep call sites terse — implementer's choice; keep it
in `theme.rs`.) Because every size already routes through `theme.rs` (HANDOFF-032),
no view needs structural change beyond swapping helper names.

> Guard: clamp effective sizes so layout stays usable; the existing scrollable
> page wrappers absorb reflow. Add a quick visual check at `Larger` for the
> densest view (Storage breakdown).

---

## 3. Reduced motion

- `reduced_motion: bool` in state; default resolved in `orbok-app`
  (`resolve_os_reduced_motion()` — best-effort: Linux portal/gsettings, Windows
  SPI_GETCLIENTAREAANIMATION, macOS `reduceMotion`; unknown → `false`).
- Thread to any animated surface. Today there are none, so this is a no-op gate:
  add the field, the setting, the message, and a `state.reduced_motion` read at
  the (future) animation site. The point is the switch exists and defaults
  correctly *before* motion is introduced.
- Rule encoded in `docs/maintainers`: new animation must check
  `state.reduced_motion`.

---

## 4. Settings surface (plain language; reuse 033 primitives)

In `views::settings_view`, render two sections (worded per GUI §23, no jargon):

```text
Appearance
  Theme:     pick_list/segmented [System|Light|Dark|High Contrast Light|High Contrast Dark]
             → Message::SetTheme(..)  (mechanism from RFC-032)
  Text size: pick_list [Default|Large|Larger]
             → Message::SetTextScale(..)

Accessibility
  [ ] Reduce motion   → Message::SetReducedMotion(bool)
  Language: [English|日本語]   (mirror existing locale control; move here)
  (info) Status colors are always shown with a label and an icon, so they stay
         clear for every kind of color vision.   ← static explanatory text
```

All controls are `components::*` (RFC-033). All strings are i18n keys (add to
en/ja). Keep the existing Advanced toggle for deeper technical detail
(progressive disclosure / "less is more").

---

## 5. Color-vision-safe status (always-on)

Extend `components::status_badge` so each tone carries a fixed lucide icon, making
status a 3-channel signal (text + icon + tone):

```rust
fn tone_icon(tone: Tone) -> char {
    use snora::lucide::*;
    char::from(match tone {
        Tone::Success => CircleCheck,
        Tone::Warning => TriangleAlert,
        Tone::Danger  => CircleX,
        Tone::Info    => Type,        // keyword
        Tone::Accent  => Sparkles,    // semantic
        Tone::Neutral => Clock,       // temporary/other
    })
}
// status_badge renders: [icon] label, tone-tinted; label & icon always present.
```

(Verify each icon constant exists in `snora::lucide`; substitute a near
equivalent if a name differs in the pinned lucide-icons version.)

CVD fixture test (§7) proves the icon+label disambiguates even when hue collapses.

---

## 6. Locale-aware formatting

Route the remaining ad-hoc `format!` display sites in views through i18n
functions (RFC-031 §5.4 centralization). Concrete sites to move:

- Storage: `format!("{gib:.3} GiB total")`, `format!("  {category}: {mib:.1} MiB ({count} items)")`,
  `format!("  {label}: {:.1} MiB", …)` → `i18n::fmt_size(locale, bytes)` /
  `i18n::fmt_count(locale, n)`.
- Search: `format!("Query: {last}")`, `"Searching…"` → i18n keys/params.
- Any other `format!` producing user-facing text in views.

Add `fmt_size`, `fmt_count`, `fmt_date` to `i18n.rs` with en/ja arms (digit
grouping per locale; this also resolves RFC-031's open "1,234 vs 1234" question
for the cases orbok renders).

---

## 7. RTL readiness (audit, mostly no code)

- Confirm `LayoutDirection` is passed to direction-aware widgets (already true
  for `app_side_bar`/`app_tab_bar` in `shell.rs`).
- Grep views for hard-coded directional layout that should be start/end; fix any
  found. Record the audit result in `accessibility.md` ("RTL is catalog-only:
  layout is direction-aware").
- No RTL locale ships now (future RFC-031 extension).

---

## 8. Persistence

`app_settings` / `OrbokSettings`: add `ui.text_scale`, `ui.reduced_motion`
(theme/locale already handled by 032/031). Load at startup, write on the persist
messages in `orbok-app` (mirror `PersistLocale`).

---

## 9. Tests

1. `text_scale_factor` + `*_scaled` helpers return expected `Pixels`;
   `ui.text_scale` round-trips.
2. `reduced_motion` persistence round-trip; mocked OS "reduce motion" → default
   `true`.
3. **CVD fixture:** apply deuteranopia/protanopia/tritanopia transforms to the
   six tone colors; assert (icon,label) pairs remain pairwise-distinct (no two
   statuses collapse to the same signal). Pure-Rust transform; no rendering.
4. `fmt_size`/`fmt_count` match expected en + ja output.
5. Direction audit heuristic test (grep) passes; direction-aware widgets get a
   `LayoutDirection`.

---

## 10. Definition of done

- [ ] Settings exposes Theme, Text size, Reduce motion, Language in plain
      language; all persist across restart.
- [ ] `Large`/`Larger` scales all text uniformly; Storage view still usable.
- [ ] Reduce motion defaults from OS; flag threaded to the (future) animation gate
      with a documented rule.
- [ ] Every status badge = text + distinct icon + tone; grayscale render still
      distinguishes all statuses (CVD fixture green).
- [ ] User-facing numbers/sizes/dates produced by i18n; en/ja correct.
- [ ] RTL-readiness audit recorded; no hard-coded left/right remains.
- [ ] Tests pass; build + tests warning-free and green.
- [ ] CHANGELOG entry under the current version (no version bump).
