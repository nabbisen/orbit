# Implementation Handoff — RFC-039: Privacy Modes and Local Data Visibility

**Project:** orbok  
**RFC:** 039  
**Implementation theme:** unified privacy modes and local-data behavior  
**Primary owners:** settings/privacy/storage/history/diagnostics/ui

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement a single privacy model that governs recent searches, temporary previews, logs, diagnostics, and local data visibility.

Modes:

```text
Standard
Strict
Portable
Diagnostics
```

## 2. Scope

### In scope

- Privacy settings model.
- Standard/Strict/Portable/Diagnostics mode behavior.
- Recent searches integration.
- Temporary previews behavior.
- Logs/diagnostics defaults.
- Storage dashboard labels.
- Strict-mode cleanup prompt.

### Out of scope

- Full encryption.
- Secure deletion guarantees.
- Enterprise policy management.
- Remote telemetry.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/core/src/privacy.rs
crates/app/src/settings.rs
crates/ui/src/screens/settings.rs
crates/ui/src/screens/storage.rs
crates/search/src/history.rs
crates/cache/src/privacy_policy.rs
crates/diagnostics/src/policy.rs
```

## 4. Data / State Changes

Add:

```rust
pub enum PrivacyMode {
    Standard,
    Strict,
    Portable,
    Diagnostics,
}
```

Add:

```rust
pub struct PrivacySettings {
    pub mode: PrivacyMode,
    pub remember_recent_searches: bool,
    pub persist_snippets: bool,
    pub clear_temporary_previews_on_exit: bool,
    pub diagnostics_include_paths: bool,
    pub diagnostics_include_recent_searches: bool,
}
```

## 5. PR Plan

### PR-039-1 — Privacy settings model

Tasks:

- Add settings schema.
- Add defaults.
- Add strict-mode override logic.

Acceptance:

- Standard defaults useful.
- Strict disables recent searches.

### PR-039-2 — Settings UI

Tasks:

- Add Privacy section.
- Add mode selector.
- Add plain explanations.
- Add strict cleanup prompt.

Acceptance:

- Users can understand what changes.

### PR-039-3 — Recent searches integration

Tasks:

- Enforce history off in strict.
- Offer clear existing history when strict enabled.

Acceptance:

- Strict mode stores no new recent searches.

### PR-039-4 — Temporary previews and storage labels

Tasks:

- Apply strict mode to snippet/preview persistence.
- Add cleanup behavior.
- Use plain storage labels.

Acceptance:

- Strict mode reduces remembered local text where practical.

### PR-039-5 — Diagnostics policy integration

Tasks:

- Expose privacy mode to diagnostics policy.
- Strict disables sensitive opt-ins by default.

Acceptance:

- Diagnostics behavior follows privacy settings.

## 6. UI Copy

```text
Documents are processed on this computer only.
Strict privacy reduces what orbok remembers.
Recent searches are saved on this computer only.
Recent searches are not saved while Strict privacy is on.
Temporary previews help results open faster. You can clear them anytime.
Your files will not be deleted.
```

## 7. Acceptance Criteria

- Privacy modes exist.
- Standard mode is useful by default.
- Strict mode reduces remembered data.
- Recent searches obey mode.
- Diagnostics obey mode.
- Temporary previews have clear cleanup behavior.
- Model download copy is accurate.
- User files are never deleted by privacy cleanup.
- Default UI uses plain language.

## 8. QA Checklist

- Standard mode default.
- Enable Strict.
- Turn on and clear.
- Verify history disabled.
- Verify temporary previews policy.
- Verify diagnostics opt-ins disabled/restricted.
- Portable mode data location copy.
- Verify no technical labels in privacy UI.
