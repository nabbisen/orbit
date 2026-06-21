# Implementation Handoff — RFC-041: Search, Narrow Results, and Browse Around

**Project:** orbok  
**RFC:** 041  
**Implementation theme:** integrated search, post-search narrowing, and result-level browse-around actions  
**Primary owners:** UI/search pipeline/app state

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement the final search UX model:

```text
Search → Narrow results → Browse around
```

The user starts with one search box and one Search button. Filters are not shown before the user searches. After results appear, orbok suggests small reversible narrowing choices. The user can later browse around a selected result without entering a separate Explorer mode.

## 2. Scope

### In scope

- Integrated search input and button.
- No pre-search filter form.
- Quick narrowing chips after results.
- Active filter chips with `×` removal.
- Clear all filters.
- More ways to narrow panel.
- Result-level actions:
  - Search in this folder;
  - Show nearby files;
  - Show similar files.
- No-results-after-filtering recovery.
- Plain user-facing labels.
- Keyboard access and focus stability.

### Out of scope

- Search history; covered by RFC-042.
- Ranking internals.
- Source refresh policy; covered by RFC-037.
- Privacy modes; covered by RFC-039.
- Diagnostics; covered by RFC-040.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/ui/src/screens/search.rs
crates/ui/src/components/filter_chip.rs
crates/ui/src/components/result_card.rs
crates/ui/src/components/more_ways_panel.rs
crates/ui/src/i18n/en.rs
crates/ui/src/i18n/ja.rs
crates/search/src/filter.rs
crates/search/src/request.rs
crates/search/src/result.rs
```

The UI owns display state. The search engine owns applying filters to the request. The UI must not know ranking internals.

## 4. Data / State Changes

Add or confirm:

```rust
pub struct SearchUiState {
    pub text: String,
    pub active_filters: Vec<ActiveFilter>,
    pub suggested_filters: Vec<SuggestedFilter>,
    pub more_panel_open: bool,
    pub results_status: ResultsStatus,
    pub selected_result_id: Option<ResultId>,
}
```

```rust
pub enum ActiveFilter {
    Folder { id: FolderId, label: String },
    Kind { id: KindId, label: String },
    Changed { value: ChangedFilter, label: String },
    ReadyStatus { value: ReadyFilter, label: String },
    SearchStyle { value: SearchStyle, label: String },
    Language { value: LanguageFilter, label: String },
}
```

Store user-facing `label` with the filter to keep active chips stable.

## 5. PR Plan

### PR-032-1 — Search UI state and filter model

Tasks:

- Add `SearchUiState` fields if missing.
- Add `ActiveFilter`, `SuggestedFilter`, `ResultsStatus` or equivalent.
- Add conversion from active filters to search request parameters.
- Add unit tests for filter add/remove/clear behavior.

Acceptance:

- Search text is preserved when filters change.
- Removing one filter removes only that filter.
- Clear removes filters but does not clear search text.

### PR-032-2 — Quick chips and active chips

Tasks:

- Add `FilterChip` component.
- Show quick chips only after results exist.
- Show at most three quick chips plus More ways.
- Show active filters under “Narrowed by”.
- Add `Clear` button when filters are active.

Acceptance:

- No filter row before first search.
- Quick chips appear after useful results.
- Active chips show `×`.
- Selected state does not rely on color alone.

### PR-032-3 — More ways to narrow panel

Tasks:

- Add panel/drawer for wide windows.
- Add inline expanded section for narrow windows if the shell cannot support a side panel.
- Add groups: Search in, Kind, Changed, Ready status.
- Add Show results and Clear.
- Escape closes panel.

Acceptance:

- Panel does not erase current result list.
- Closing panel does not clear filters.
- Keyboard navigation order is predictable.

### PR-032-4 — No-results recovery

Tasks:

- Add EmptyAfterFiltering state.
- Show active filter chips with removal options.
- Show Clear action.
- Avoid blank result list.

Acceptance:

- No “0 results” dead end.
- User can recover with keyboard or mouse.

### PR-032-5 — Browse-around actions

Tasks:

- Add result preview actions:
  - Open file;
  - Search in this folder;
  - Show nearby files;
  - Show similar files.
- Implement `SearchInResultFolder` first.
- Gate `ShowSimilarFiles` if meaning search is unavailable.

Acceptance:

- No top-level Explorer tab is introduced.
- Result-level actions are visible after selecting a result.
- Search in this folder adds a folder filter.

## 6. UI Copy

Required copy:

```text
Search your files...
Search
Narrow results
Narrowed by
More ways to narrow
Clear
No results with these choices
Try removing one.
Search in this folder
Show nearby files
Show similar files
```

Do not use in default UI:

```text
query
source
index
cache
vector
embedding
RRF
BM25
```

## 7. Acceptance Criteria

- Default search screen has no filter form before search.
- Results show quick narrowing only when useful.
- Active filters are visible and removable.
- Clear does not clear search text.
- More ways to narrow works.
- No-result-after-filtering state gives recovery actions.
- Browse-around actions are available from selected results.
- Default labels avoid technical terms.
- Keyboard and focus behavior pass manual QA.

## 8. QA Checklist

- Search with no folders.
- Add folder, search, verify chips appear.
- Apply folder chip.
- Apply kind chip.
- Remove one chip.
- Clear all chips.
- Use More ways panel.
- Produce no results by over-filtering and recover.
- Select result and use Search in this folder.
- Verify Search box focus remains stable.
- Verify default UI contains no forbidden technical labels.
