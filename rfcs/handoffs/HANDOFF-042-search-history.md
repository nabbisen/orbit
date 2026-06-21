# Implementation Handoff — RFC-042: Search History and Reopen Recent Searches

**Project:** orbok  
**RFC:** 042  
**Implementation theme:** local recent searches without automatic result tabs  
**Primary owners:** UI/search state/settings/privacy/storage

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement local search history as:

```text
Recent searches → Search again
```

Do not implement automatic search-result tabs. Reopened searches restore search words and narrowing choices, then run again against current files.

## 2. Scope

### In scope

- Store recent successful searches locally.
- Restore search words and filters.
- Re-run against current files.
- Deduplicate identical searches.
- Clear recent searches.
- Remember recent searches setting.
- Strict privacy integration hook.
- Redacted logs and diagnostics behavior hooks.

### Out of scope

- Automatic result tabs.
- Saved searches workspace.
- Cloud sync.
- Search alerts.
- Full result snapshot persistence.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/ui/src/components/recent_searches.rs
crates/ui/src/screens/search.rs
crates/ui/src/screens/settings.rs
crates/search/src/history.rs
crates/data/catalog/search_history_repo.rs
crates/core/src/privacy.rs
```

Search history stores instructions, not frozen results.

## 4. Data / State Changes

Add persisted entry:

```rust
pub struct SearchHistoryEntry {
    pub id: SearchHistoryId,
    pub search_text: String,
    pub filters: Vec<StoredSearchFilter>,
    pub created_at: Timestamp,
    pub last_used_at: Timestamp,
    pub previous_result_count: Option<usize>,
    pub locale: Locale,
}
```

Add setting:

```rust
pub struct SearchHistorySettings {
    pub remember_recent_searches: bool,
    pub max_entries: usize,
    pub clear_when_privacy_strict: bool,
}
```

Default `max_entries = 20`.

## 5. PR Plan

### PR-033-1 — Storage and repository

Tasks:

- Add history table or settings-backed storage.
- Implement create/update/deduplicate/list/clear.
- Do not store snippets or full result lists.
- Enforce maximum count.

Acceptance:

- Empty searches are not stored.
- Duplicate search+filters update timestamp instead of duplicating.
- History remains local.

### PR-033-2 — Search result integration

Tasks:

- After successful search, create/update recent entry if enabled.
- Store active filters as narrowing choices.
- Do not store failed searches.
- Preserve zero-result searches only if policy says yes; keep them low prominence.

Acceptance:

- Successful searches appear in recent list.
- Disabled history stores nothing.

### PR-033-3 — Recent searches UI

Tasks:

- Add recent searches list on empty/idle search screen.
- Add compact Recent searches button on results screen.
- Add drawer/panel if needed.
- Use “Search again” action.

Acceptance:

- No automatic tabs appear.
- Recent entries are keyboard focusable.

### PR-033-4 — Reopen behavior

Tasks:

- Restore search text immediately.
- Restore valid filters.
- Drop invalid filters with friendly notice.
- Run search again.
- Update `last_used_at`.

Acceptance:

- Reopened search uses current files, not stale snapshots.
- Missing folder filter is dropped safely.

### PR-033-5 — Settings and privacy

Tasks:

- Add Remember recent searches toggle.
- Add Clear recent searches action.
- Add confirmation for clear-all.
- Wire strict privacy mode to disable history.

Acceptance:

- Strict privacy disables history.
- Clear history does not delete files or search data.

## 6. UI Copy

Required copy:

```text
Recent searches
Search again
Clear recent searches
Remember recent searches
Recent searches are saved on this computer only.
Recent searches are not saved while Strict privacy is on.
```

Clear confirmation:

```text
Clear recent searches?
This removes the list of searches shown in orbok.
Your files and search data are not deleted.
```

## 7. Acceptance Criteria

- Users can reopen recent searches.
- Reopened searches rerun against current files.
- Search text and narrowing choices are restored.
- Invalid filters are safely dropped with friendly copy.
- User can clear recent searches.
- User can turn history off.
- Strict privacy disables history.
- No default result tabs.
- No search text in logs or diagnostics by default.

## 8. QA Checklist

- Search, close app, reopen, verify recent search.
- Reopen search and verify current result refresh.
- Search with filters, reopen, verify filters restored.
- Remove folder, reopen old search, verify dropped-filter notice.
- Turn history off and verify no new entries.
- Clear recent searches and verify list is empty.
- Enable strict privacy and verify history disabled.
- Inspect default diagnostics/logs for search text absence.
