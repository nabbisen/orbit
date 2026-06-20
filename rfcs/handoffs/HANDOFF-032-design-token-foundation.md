# HANDOFF-032 — Design Token Foundation and Theming

**RFC:** `rfcs/proposed/032-design-token-foundation-and-theming.md`
**Owner crate(s):** `orbok-ui` (primary), `orbok-app` (persistence + OS probe)
**Prereqs:** none (this is the foundation). **Blocks:** 033, 034, 035.
**Release:** current version — do **not** bump.

---

## 0. One-paragraph orientation

`AppState` already holds `tokens: snora::design::Tokens`, but it only feeds the
notice primitive. This work threads tokens into **every** view, deletes all
magic-number `.size()`/`.padding()`/`.spacing()` and any literal colors in view
code, replaces the `high_contrast: bool` with a five-value `Theme`, and persists
the theme like the locale. After this, RFC-033 can swap in primitives and
RFC-034 can rely on token contrast.

---

## 1. Files

| File | Action |
|---|---|
| `crates/ui/src/theme.rs` | **new** — `Theme` enum + typography/spacing helper fns |
| `crates/ui/src/lib.rs` | add `pub mod theme;` re-export `Theme`; fix stale "snora 0.8" doc comment |
| `crates/ui/src/state.rs` | replace `high_contrast: bool` → `theme: Theme`; add `SetTheme`/`PersistTheme`; recompute `tokens` on change |
| `crates/ui/src/views.rs` | replace every literal size/padding/spacing with `theme::*` helpers + `tokens.spacing.*`; remove `heading()` magic `26` |
| `crates/ui/src/views/wizard.rs` | same literal sweep |
| `crates/ui/src/shell.rs` | same literal sweep (any sizes/padding) |
| `crates/ui/src/notice.rs` | already token-aware; verify no literals remain |
| `crates/app/src/bootstrap.rs` | load `ui.theme`; resolve `System`→concrete via OS probe; build initial `tokens` |
| `crates/app/src/settings.rs` | add `theme` field to `OrbokSettings` (mirror `locale`) |
| `crates/app/src/main.rs` | wire `PersistTheme` → catalog/settings write (mirror `PersistLocale`) |
| `crates/ui/src/tests.rs` | replace `high_contrast_toggle_*` test with `SetTheme` tests |
| `docs/src/users/settings.md` | document theme options |
| `docs/src/maintainers/architecture.md` | note the token-gateway rule |

---

## 2. `theme.rs` (new)

```rust
//! orbok theme selection and token-reading helpers (RFC-032).
//!
//! The single place the UI reads design *values*. Views never use literal
//! sizes/paddings/colors; they call these helpers, which read the active
//! `snora::design::Tokens`.

use iced::Pixels;
use serde::{Deserialize, Serialize};
use snora::design::Tokens;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::HighContrastLight => "high_contrast_light",
            Theme::HighContrastDark => "high_contrast_dark",
        }
    }
    pub fn parse(s: &str) -> Option<Theme> {
        Some(match s {
            "system" => Theme::System,
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "high_contrast_light" => Theme::HighContrastLight,
            "high_contrast_dark" => Theme::HighContrastDark,
            _ => return None,
        })
    }
    /// Concrete bundle. `System` must be pre-resolved by `orbok-app`; if it
    /// reaches here, fall back to Light (safe default).
    pub fn tokens(self) -> Tokens {
        match self {
            Theme::Light | Theme::System => Tokens::light(),
            Theme::Dark => Tokens::dark(),
            Theme::HighContrastLight => Tokens::high_contrast_light(),
            Theme::HighContrastDark => Tokens::high_contrast_dark(),
        }
    }
    pub const ALL: &'static [Theme] = &[
        Theme::System, Theme::Light, Theme::Dark,
        Theme::HighContrastLight, Theme::HighContrastDark,
    ];
}

// ── Typography helpers (wrap the snora style bridge) ──────────────────
pub fn heading(t: &Tokens) -> Pixels { snora::design::style::text::heading_size(t) }
pub fn title(t: &Tokens)   -> Pixels { snora::design::style::text::title_size(t) }
pub fn body(t: &Tokens)    -> Pixels { snora::design::style::text::body_size(t) }
pub fn meta(t: &Tokens)    -> Pixels { snora::design::style::text::body_small_size(t) }
pub fn label(t: &Tokens)   -> Pixels { snora::design::style::text::label_size(t) }
```

> Note: `theme::heading` replaces the local `fn heading(label)` in `views.rs`.
> Keep a `heading_text(tokens, s)` convenience if useful, but the **size** comes
> from the token.

---

## 3. Literal → token replacement map (apply across all view modules)

Font sizes:

| Found | Replace with |
|---|---|
| `.size(26)` | `.size(theme::heading(tokens))` |
| `.size(22)` / `.size(20)` | `.size(theme::title(tokens))` |
| `.size(18)` | `.size(theme::title(tokens))` |
| `.size(15)` / `.size(14)` / `.size(13)` | `.size(theme::body(tokens))` |
| `.size(12)` / `.size(11)` | `.size(theme::meta(tokens))` |
| button label sizes | `.size(theme::label(tokens))` |

Spacing / padding:

| Found | Replace with |
|---|---|
| `.spacing(2)` | `.spacing(tokens.spacing.xs)` |
| `.spacing(4)` / `(6)` / `(8)` | `.spacing(tokens.spacing.sm)` |
| `.spacing(10)` | `.spacing(tokens.spacing.md)` |
| `.padding(10)` / `(8)` | `.padding(tokens.spacing.md)` / `.sm` |
| `Padding::from([12.0,16.0])` | `Padding::from([tokens.spacing.md, tokens.spacing.lg])` |
| `Padding::from([12.0,18.0])` | `Padding::from([tokens.spacing.md, tokens.spacing.lg])` |
| `page()` `Padding::from([28.0,40.0])` | `Padding::from([tokens.spacing.xl, tokens.spacing.xxl])` |

Colors: there are currently **no literal colors** in view code (status is text,
not color) — good. Add the CI gate anyway (§6) to keep it that way.

`tokens` in each view = `&state.tokens`. Helper fns that build sub-elements take
`tokens: &Tokens` explicitly (so RFC-033 can unit-test them).

---

## 4. `state.rs` changes

- Remove field `high_contrast: bool`; add `theme: Theme`. Keep `tokens`.
- `Default`: `theme: Theme::default()` (System), `tokens: Tokens::light()`
  (app overrides after OS resolve).
- Remove `Message::ToggleHighContrast`; add:
  ```rust
  SetTheme(Theme),        // user picked a theme in Settings
  PersistTheme(Theme),    // emitted for orbok-app to persist (mirrors PersistLocale)
  ```
- `update`:
  ```rust
  Message::SetTheme(theme) => {
      self.theme = *theme;
      self.tokens = theme.tokens(); // System pre-resolved by app before send
  }
  // PersistTheme is handled in orbok-app, like PersistLocale.
  ```

> The Settings view currently renders a high-contrast toggle button
> (`views.rs` ~line 470). Replace it in **RFC-035's** Settings work with the
> full theme picker; for **this** RFC, render a minimal theme picker (5 buttons
> or a pick_list) so the feature is reachable and testable now.

---

## 5. `orbok-app` changes

- `OrbokSettings` (settings.rs): add `pub theme: String` (default `"system"`),
  exactly mirroring `locale`.
- `bootstrap.rs`: after loading settings, resolve theme:
  ```rust
  let stored = Theme::parse(&settings.theme).unwrap_or_default();
  let resolved = match stored {
      Theme::System => resolve_os_theme(), // -> Theme::Dark | Theme::Light
      other => other,
  };
  let tokens = resolved.tokens();
  // seed AppState.theme = stored (keep "System" as the stored intent),
  // AppState.tokens = tokens (the resolved concrete bundle)
  ```
- `resolve_os_theme()` (new, in orbok-app — platform I/O belongs here, not in
  orbok-ui): best-effort OS color-scheme probe. Linux: `XDG_*`/desktop portal or
  `dark`/`light` heuristic; Windows: registry `AppsUseLightTheme`; macOS:
  `AppleInterfaceStyle`. Unknown → `Theme::Light`. Keep it small and
  feature-gated/best-effort; a wrong guess is non-fatal (user can override).
- `main.rs`: handle `PersistTheme(theme)` → write `ui.theme` to catalog settings
  (mirror the existing `PersistLocale` / `set("ui.locale", …)` path at
  bootstrap.rs ~line 212).

---

## 6. CI grep gate (heuristic, mirrors RFC-031's literal gate)

Add to `scripts/` (or `xtask`) a check that fails if view/component modules
contain forbidden literals:

```sh
# scripts/check-design-tokens.sh
set -e
VIEW_GLOB='crates/ui/src/views.rs crates/ui/src/views/*.rs crates/ui/src/shell.rs crates/ui/src/components.rs'
# numeric .size( / .padding( with integer/float literal, and any iced color literal
if grep -nE '\.size\([0-9]' $VIEW_GLOB 2>/dev/null; then echo "literal text size in view"; exit 1; fi
if grep -nE '\.padding\([0-9]' $VIEW_GLOB 2>/dev/null; then echo "literal padding in view"; exit 1; fi
if grep -nE 'iced::Color|Color::from_rgb|from_rgba' $VIEW_GLOB 2>/dev/null; then echo "literal color in view"; exit 1; fi
echo "design-token gate: ok"
```

Wire it into the existing CI workflow and document it in
`docs/src/maintainers/development.md`. (`components.rs` does not exist until
RFC-033; the glob tolerates its absence.)

---

## 7. Tests (`crates/ui/src/tests.rs`)

1. `theme_tokens_match_preset`: for each non-`System` `Theme`, assert
   `theme.tokens()` equals the corresponding `Tokens::*()` preset.
2. `set_theme_updates_tokens`: `SetTheme(Dark)` → `state.tokens == Tokens::dark()`.
3. `theme_string_roundtrip`: `Theme::parse(t.as_str()) == Some(t)` for all.
4. `os_theme_resolves` (in orbok-app tests): mock env → dark yields a dark
   theme, unknown yields Light.
5. Delete `high_contrast_toggle_swaps_token_preset`; its intent is covered by
   (2) using `SetTheme(HighContrastLight)`.

Run: backend + `cargo test -p orbok-ui`; both green, zero warnings.

---

## 8. Definition of done

- [ ] `theme.rs` added; `Theme` reachable from `orbok-ui` root.
- [ ] Zero magic numbers in `views.rs`, `views/wizard.rs`, `shell.rs`
      (grep gate passes).
- [ ] `high_contrast` removed; `theme` + `tokens` drive styling.
- [ ] Theme picker reachable in Settings; selection persists across restart.
- [ ] `System` resolves to dark on a dark OS, light otherwise.
- [ ] Stale "snora 0.8" doc comments in `lib.rs`/`shell.rs` corrected to 0.25.
- [ ] Tests above pass; `cargo build --workspace` and `cargo test --workspace`
      warning-free and green.
- [ ] CHANGELOG entry under the current version (no version bump).
