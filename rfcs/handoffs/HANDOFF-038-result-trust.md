# Implementation Handoff — RFC-038: Result Freshness, Trust Badges, and Recovery Actions

**Project:** orbok  
**RFC:** 038  
**Implementation theme:** trustworthy result status and recovery actions  
**Primary owners:** search result builder/ui/extraction warning mapping/source lifecycle

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Every result must be honest about whether it is ready, changed, missing, still preparing, partly prepared, or cannot be opened.

Default UI should stay clean: show badges only when trust-relevant.

## 2. Scope

### In scope

- `ResultTrustState`.
- Mapping from file/source/extraction states to trust state.
- Trust badges.
- Recovery actions.
- Warnings from RFC-035 mapped to plain copy.
- Ready status filter support.

### Out of scope

- Ranking formula changes beyond mild guidance.
- Extraction internals.
- Diagnostics export.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/search/src/result_trust.rs
crates/search/src/result.rs
crates/ui/src/components/result_badge.rs
crates/ui/src/components/result_preview.rs
crates/ui/src/i18n/en.rs
crates/ui/src/i18n/ja.rs
crates/fs/src/source_lifecycle.rs
```

## 4. Data / State Changes

Add:

```rust
pub enum ResultTrustState {
    Ready,
    NeedsUpdate,
    FileNotFound,
    StillBeingPrepared,
    PartlyPrepared,
    CannotOpen,
}
```

Add:

```rust
pub struct SearchResultTrust {
    pub state: ResultTrustState,
    pub warnings: Vec<ResultWarningSummary>,
    pub recovery_actions: Vec<ResultRecoveryAction>,
}
```

## 5. PR Plan

### PR-038-1 — Trust computation

Tasks:

- Compute trust from file state, source state, extraction warnings, and open status.
- Add unit tests.

Acceptance:

- Changed file → NeedsUpdate.
- Missing file → FileNotFound.
- Extraction warning → PartlyPrepared.

### PR-038-2 — Result card badges

Tasks:

- Add text badges.
- Hide Ready badge by default.
- Show trust-relevant badges.
- Ensure no color-only status.

Acceptance:

- Clean ready results remain uncluttered.

### PR-038-3 — Preview recovery actions

Tasks:

- Add actions by state:
  - Prepare again;
  - Check folder;
  - Remove from results;
  - Open anyway;
  - Show in folder;
  - View details.

Acceptance:

- Every non-ready result has an understandable next step.

### PR-038-4 — More ways ready-status filter

Tasks:

- Add Ready status filter group.
- Include Ready, Needs update, File not found, Partly prepared.

Acceptance:

- User can narrow by readiness in More ways.

### PR-038-5 — Extraction warning details

Tasks:

- Map RFC-035 warnings to plain details.
- Show only important warning in default UI.
- Advanced view shows more detail.

Acceptance:

- Scanned/no-text PDF is explained without technical jargon.

## 6. UI Copy

```text
Needs update
File not found
Still being prepared
Partly prepared
Cannot open
This file changed after orbok prepared it.
Only part of this file was prepared.
Some pages could not be prepared.
This PDF may contain images instead of selectable text.
```

## 7. Acceptance Criteria

- Trust states are explicit.
- Badges show only when useful.
- Missing files do not look ready.
- Changed files show Needs update.
- Partly prepared files remain searchable but honest.
- Recovery actions exist.
- Status is not color-only.
- Default labels are plain.

## 8. QA Checklist

- Ready result has no badge.
- Edit file after indexing.
- Delete file after indexing.
- Disconnect source folder.
- Trigger extraction warning.
- Trigger permission error.
- Use recovery actions.
- Verify Advanced view details.
- Verify screen reader/focus behavior for badges/actions.
