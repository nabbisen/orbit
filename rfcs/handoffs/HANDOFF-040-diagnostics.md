# Implementation Handoff — RFC-040: Safe Diagnostics and Redacted Support Bundle

**Project:** orbok  
**RFC:** 040  
**Implementation theme:** supportability without leaking private documents or search intent  
**Primary owners:** diagnostics/settings/logging/privacy/storage

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement a manual support-file export that is redacted by default.

Default support file must not include:

```text
document contents
search words
raw local paths
snippets
embeddings
full database
model files
```

## 2. Scope

### In scope

- Manual diagnostics export flow.
- Preview before export.
- Redacted bundle contents.
- Manifest with inclusion flags.
- Redacted logs.
- Privacy-mode integration.
- Optional sensitive opt-ins disabled by default.

### Out of scope

- Automatic telemetry.
- Automatic upload.
- Full database export.
- Document upload.
- Model file export.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/diagnostics/src/lib.rs
crates/diagnostics/src/collector.rs
crates/diagnostics/src/redaction.rs
crates/diagnostics/src/export.rs
crates/ui/src/screens/settings.rs
crates/core/src/privacy.rs
crates/logging/src/redaction.rs
```

## 4. Data / State Changes

Add:

```rust
pub struct DiagnosticsPolicy {
    pub include_raw_paths: bool,
    pub include_folder_names: bool,
    pub include_recent_searches: bool,
    pub include_detailed_logs: bool,
    pub privacy_mode: PrivacyMode,
}
```

Add bundle manifest:

```json
{
  "app": "orbok",
  "bundle_version": 1,
  "redacted": true,
  "includes_document_contents": false,
  "includes_search_text": false,
  "includes_raw_paths": false
}
```

## 5. PR Plan

### PR-040-1 — Diagnostics collectors

Tasks:

- App/platform/settings/storage/source/indexing/extraction/model/scheduler/error collectors.
- Return structured sections.
- No raw sensitive values by default.

Acceptance:

- Default collectors produce useful summary.

### PR-040-2 — Redaction engine

Tasks:

- Redact absolute paths.
- Redact search text markers.
- Redact URL query tokens.
- Redact environment-like secrets.
- Add unit tests.

Acceptance:

- Raw paths and search text absent by default.

### PR-040-3 — Bundle exporter

Tasks:

- Create zip or tar.gz.
- Include manifest and summary.
- Include redacted JSON sections and logs.
- Handle partial collector failure.

Acceptance:

- Bundle is created locally and can be inspected.

### PR-040-4 — UI flow

Tasks:

- Settings → Create support file.
- Preview included/excluded data.
- Optional More details section.
- Export result screen with Show file / Done.

Acceptance:

- No silent upload.
- User sees what is included.

### PR-040-5 — Privacy integration

Tasks:

- Respect Strict mode.
- Sensitive opt-ins default off.
- Manifest records opt-ins.

Acceptance:

- Strict mode exports minimal redacted bundle.

## 6. UI Copy

```text
Create a support file
This file helps diagnose problems.
It does not include your documents or search words by default.
Not included: Documents, Search words, Raw folder paths
Support file created.
Support file was not created. Please choose another location or try again.
```

## 7. Acceptance Criteria

- User can create support file manually.
- Default bundle is redacted.
- Documents excluded.
- Search text excluded.
- Raw paths excluded by default.
- Manifest states inclusion flags.
- Preview shows included/excluded data.
- Sensitive opt-ins off by default.
- Strict privacy restricts diagnostics.
- Redaction tests exist.
- No automatic upload.

## 8. QA Checklist

- Create default support file.
- Open archive and inspect contents.
- Search for raw home path/user name.
- Search for known query string.
- Verify no snippets/document text.
- Enable folder-name opt-in and inspect.
- Enable strict privacy and export.
- Simulate missing logs.
- Simulate collector failure.
- Verify cancel path.
