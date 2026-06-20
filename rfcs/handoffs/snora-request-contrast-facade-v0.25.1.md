# snora request — re-export `contrast` under `snora::design` (target: v0.25.1)

**To:** snora maintainers
**From:** nabbisen (orbok)
**Type:** Facade consistency fix (additive, non-breaking)
**Target release:** v0.25.1 (patch)
**Affected crate:** `snora` (the facade). No change to `snora-design` or `snora-widgets`.

---

## The ask

Re-export the `contrast` module under the `snora::design` facade, the same way
the other Snora Design sub-modules are already surfaced, so that
`snora::design::contrast::{relative_luminance, contrast_ratio, composite_over}`
resolves when the `design` feature is enabled.

## Why

`snora::design` is the intended single entry point for the Snora Design system:
downstream apps are steered through the facade rather than naming
`snora-design`/`snora-widgets` directly. The facade already re-exports the token
types and **five** sibling modules — `style`, `button`, `card`, `notice`,
`chip`, `progress` — but `contrast` was omitted. It is the only public
`snora-design` module not reachable through `snora::design`.

That gap forces an app that wants the contrast utilities to add a *second*,
otherwise-unnecessary direct dependency on `snora-design`, which undercuts the
facade pattern and couples the app to an inner crate it is meant to reach only
through `snora`.

Concrete use case: orbok's accessibility work (RFC-034, WCAG 2.1 AA) adds a
contrast **usage-guard test** — it enumerates the foreground/background palette
role pairs the app actually renders and asserts each meets the AA ratio across
all four presets, using `contrast_ratio`. This is exactly the "applications can
reuse them to check their own custom tokens" scenario the `contrast` module's
own docs invite. Everything else that test needs (`Tokens`, `Color`, `Palette`)
is already on the facade; only `contrast` is missing.

## The gap (precise)

In `snora/src/lib.rs`, the `pub mod design { … }` block re-exports the token
types plus `style`, `button`, `card`, `notice`, `chip`, `progress` — but not
`contrast`. `snora-design` itself already exposes
`pub mod contrast` with `relative_luminance`, `contrast_ratio`, and
`composite_over`; this request only surfaces it through the facade.

## Proposed change

Add a `contrast` re-export inside `pub mod design`, mirroring the existing
`pub mod style` / `pub mod button` / … pattern:

```rust
// snora/src/lib.rs, inside `pub mod design { … }`

/// Pure-Rust WCAG contrast utilities (relative luminance, contrast ratio,
/// alpha compositing), re-exported from `snora-design`.
pub mod contrast {
    pub use snora_design::contrast::{composite_over, contrast_ratio, relative_luminance};
}
```

(Equivalently `pub use snora_design::contrast;` next to the token-type
re-export; the `pub mod` form is preferred for consistency with the other
sibling modules and to keep the re-export list explicit.)

`snora-design` is already a dependency of `snora` under the `design` feature, so
no new dependency, feature, or version constraint is introduced.

## Why this is a patch (v0.25.1), not a minor

- **Additive only.** No existing path, signature, type, or behavior changes.
  Nothing that compiles today stops compiling.
- **Capability already public.** The functions are already part of the public
  API of the published `snora-design` crate; the ecosystem gains no *new*
  capability — only the intended facade path that was inadvertently left off.
- **Closes an unintended omission**, i.e. a completeness/consistency defect in
  the facade, rather than adding a feature. That framing fits a patch bump.

If the team prefers to treat any new facade path as minor, this can ship in
0.26.0 instead; from orbok's side either is fine, but 0.25.1 is the smallest,
fastest fix and is what we'd recommend.

## Suggested inclusion in the release

1. The re-export above.
2. One doc line in the `snora::design` module docs listing `contrast` alongside
   `style` in the "Exposes:" bullet list, so the facade docs stay accurate.
3. A trivial smoke test that `snora::design::contrast::contrast_ratio(black,
   white)` returns ~21.0 (guards the re-export against future accidental
   removal). The math itself is already covered by `snora-design`'s own suite.

## Acceptance criteria

- With the `design` feature enabled,
  `use snora::design::contrast::{relative_luminance, contrast_ratio, composite_over};`
  compiles in a downstream crate.
- `snora::design` module docs mention `contrast`.
- No other public API path changes; existing tests unchanged and green.

---

### Note — separate, larger asks (not part of this patch)

orbok's design-system RFCs flagged three further upstream candidates that are
**not** part of this v0.25.1 request and would each need their own design
review and a minor/feature release: a `badge` primitive (a tone-styled status
pill, currently composed by hand from the chip bridge), a modal/dialog
primitive, and a split-pane primitive. We'll raise those separately with their
own proposals; calling them out here only so this patch stays scoped to the
one-line contrast re-export.
