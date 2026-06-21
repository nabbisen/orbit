# orbok — Search History External Design Specification

**Version:** 0.1  
**Date:** 2026-06-18  
**Target application:** orbok  
**Former project name:** orbit  
**Target users:** non-technical desktop users, professionals, researchers, developers  
**Target platforms:** Linux, Windows, macOS desktop  
**Framework direction:** Rust · iced 0.14 · snora  

---

## 1. Design Conclusion

The accepted UX direction is:

```text
Search
→ Narrow results
→ Recent searches for reopening
→ Optional kept searches later
```

Search history should exist, but it should not create automatic result tabs by default.

The default design is:

```text
One active search screen
+ Recent searches list
+ privacy setting
+ clear history action
```

The app should not show:

```text
Search tab 1
Search tab 2
Search tab 3
```

by default.

Tabs can be powerful, but for non-technical users they introduce a second mental model: users must understand which tab is active, whether results are old, whether filters are shared, and whether closing a tab loses anything.

Therefore, **orbok keeps search history as a simple reopen list, not as automatic search-result tabs.**

---

## 2. Product Principle

A user should feel:

```text
I searched before.
orbok remembers it on this computer.
I can reopen it if I need it.
I can clear it anytime.
```

A user should not feel:

```text
I have multiple search workspaces and I do not know which one I am using.
```

Search history should reduce effort, not create workspace management.

---

## 3. Scope

## 3.1. In Scope

This design covers:

- recent search history;
- reopening a previous search;
- restoring search words and narrowing choices;
- refreshing results against current files;
- privacy setting for search history;
- clearing history;
- empty states;
- error and recovery states;
- future “Keep this search” path;
- why automatic tab UI is not the default;
- text wireframes;
- implementation-facing state model.

## 3.2. Out of Scope

This design does not cover:

- full saved-search automation;
- background scheduled searches;
- cloud sync;
- multi-device history;
- collaborative search;
- chat history;
- browser-like tab management;
- search result pinboards;
- history analytics.

---

## 4. User-Facing Terminology

## 4.1. Required Labels

| Concept | User-facing label |
|---|---|
| search history | Recent searches |
| saved search | Kept search |
| stored query | Saved search words |
| filters | Narrowing choices |
| search result snapshot | Previous result list |
| re-run query | Search again |
| clear history | Clear recent searches |
| disable history | Remember recent searches |

## 4.2. Avoid These Labels

Do not show these in the default UI:

```text
query
snapshot
session
workspace
tab
state
persistence
cache
database
replay
rehydrate
```

---

## 5. Design Decision: History Yes, Tabs No

## 5.1. Why Keep History

Search history helps users who:

- repeat the same search over days;
- forget exact phrasing;
- compare previous and current files;
- return to research after interruption;
- want to recover after closing the app.

It also fits the Search → Narrow results workflow because history can reopen:

- search text;
- selected folders;
- selected kinds;
- changed-date choice;
- search style if Advanced view was used.

## 5.2. Why Avoid Tabs by Default

Automatic search tabs create complexity:

```text
Which search tab am I using?
Are these results current?
Did I change the same filter in another tab?
Can I close this?
Will closing lose my search?
```

For non-technical users, that is too much surface area.

## 5.3. Future Direction

A future version may add:

```text
Keep this search
```

This creates intentional saved searches. It is better than automatic tabs because the user chooses what is worth keeping.

---

## 6. History Behavior

## 6.1. What Is Stored

A recent search entry should store:

- search words;
- active narrowing choices;
- search style if applicable;
- selected folder choices;
- selected kind choices;
- changed-date choice;
- timestamp;
- optional result count from the previous run;
- optional selected result ID, if safe and still valid.

It should not store full result snippets by default.

## 6.2. What Is Not Stored by Default

Do not store by default:

- full document text;
- result snippets;
- document contents;
- raw internal scores;
- internal ranking details;
- model/vector details.

## 6.3. Reopening Behavior

When a user clicks a recent search:

1. restore search words;
2. restore narrowing choices;
3. show a short “Searching again...” state;
4. run the search against current files;
5. show current results;
6. if files changed, show a friendly notice if needed.

Do not simply show stale old results as if they are current.

## 6.4. Result Refresh Rule

Recent searches are not permanent result snapshots.

They are remembered search instructions.

When reopened, orbok should search again.

---

## 7. Privacy Rules

Search history can reveal sensitive intent.

Therefore:

- history must be local-only;
- user must be able to turn it off;
- user must be able to clear it;
- strict privacy mode should turn it off;
- diagnostics export must not include recent searches by default.

Default setting recommendation:

```text
Remember recent searches: On
```

Reason:

- useful in a local desktop app;
- low surprise if clearly explained;
- user can turn off.

Required copy:

```text
Recent searches are saved on this computer only.
```

If privacy mode is strict:

```text
Recent searches are not saved.
```

---

## 8. Information Architecture

Recent searches belong on the Search screen.

They may appear in one of three places:

1. empty state before typing;
2. small button near the search input;
3. optional drawer/panel.

Recommended default:

```text
[Recent searches]
```

near the search input after at least one search exists.

Recent searches should not be a top-level navigation item.

---

## 9. Screen Wireframes

## 9.1. Search Screen With No History

```text
┌────────────────────────────────────────────────────────────────────┐
│  Sidebar       │ Search                                             │
│                ├────────────────────────────────────────────────────┤
│  🔍 Search     │                                                    │
│  🧠 Better     │  ┌──────────────────────────────────────────────┐  │
│     search     │  │ Search your files...                          │  │
│  ⚙ Settings   │  └──────────────────────────────────────────────┘  │
│                │                                      [Search]      │
│                │                                                    │
│                │  Nothing to search yet                            │
│                │  Choose a folder, and orbok will prepare it        │
│                │  for search. Your files stay on this computer.     │
│                │                                                    │
│                │  [Choose a folder]                                │
│                │                                                    │
└────────────────────────────────────────────────────────────────────┘
```

No history section appears because there are no recent searches.

---

## 9.2. Search Screen With Recent Searches

```text
┌────────────────────────────────────────────────────────────────────┐
│  Sidebar       │ Search                                             │
│                ├────────────────────────────────────────────────────┤
│  🔍 Search     │                                                    │
│  🧠 Better     │  ┌──────────────────────────────────────────────┐  │
│     search     │  │ Search your files...                          │  │
│  ⚙ Settings   │  └──────────────────────────────────────────────┘  │
│                │                                      [Search]      │
│                │                                                    │
│                │  Recent searches                                  │
│                │  ┌──────────────────────────────────────────────┐  │
│                │  │ authentication token rotation                  │  │
│                │  │ PDFs · Documents · 10 minutes ago              │  │
│                │  └──────────────────────────────────────────────┘  │
│                │  ┌──────────────────────────────────────────────┐  │
│                │  │ 監査 証跡 ログ                                  │  │
│                │  │ All folders · yesterday                        │  │
│                │  └──────────────────────────────────────────────┘  │
│                │                                                    │
│                │  [Clear recent searches]                          │
│                │                                                    │
└────────────────────────────────────────────────────────────────────┘
```

Rules:

- show only a small number, such as 3 to 5 recent searches;
- avoid overwhelming the empty Search screen;
- each item is clickable;
- each item shows plain summary of narrowing choices;
- timestamps use friendly wording.

---

## 9.3. Search Results With Recent Button

```text
┌────────────────────────────────────────────────────────────────────┐
│  Sidebar       │ Search                                             │
│                ├────────────────────────────────────────────────────┤
│  🔍 Search     │  ┌──────────────────────────────────────────────┐  │
│  🧠 Better     │  │ authentication token rotation                  │  │
│     search     │  └──────────────────────────────────────────────┘  │
│  ⚙ Settings   │                                      [Search]      │
│                │                                                    │
│                │  [Recent searches]                                │
│                │                                                    │
│                │  36 results                                       │
│                │  Narrow results                                   │
│                │  [PDFs] [This folder] [Changed recently] [More]   │
│                │                                                    │
│                │  Result card                                      │
│                │  Result card                                      │
│                │                                                    │
└────────────────────────────────────────────────────────────────────┘
```

Rules:

- when results are shown, do not show the full history list inline;
- show a compact Recent searches button;
- opening it should use a drawer or panel.

---

## 9.4. Recent Searches Drawer

```text
┌────────────────────────────────────────────────────────────────────┐
│  Search results                         │ Recent searches           │
│                                         │                            │
│  Search your files...        [Search]   │ authentication token       │
│                                         │ rotation                   │
│  36 results                             │ PDFs · Documents           │
│                                         │ 10 minutes ago             │
│  Result card                            │ [Search again]             │
│  Result card                            │                            │
│                                         │ 監査 証跡 ログ              │
│                                         │ All folders                │
│                                         │ yesterday                  │
│                                         │ [Search again]             │
│                                         │                            │
│                                         │ [Clear recent searches]    │
└────────────────────────────────────────────────────────────────────┘
```

Rules:

- the current result list remains visible;
- the drawer does not become a separate workspace;
- “Search again” is clearer than “Open” or “Restore.”

---

## 9.5. Reopening a Recent Search

```text
┌────────────────────────────────────────────────────────────────────┐
│  Sidebar       │ Search                                             │
│                ├────────────────────────────────────────────────────┤
│  🔍 Search     │  ┌──────────────────────────────────────────────┐  │
│  🧠 Better     │  │ authentication token rotation                  │  │
│     search     │  └──────────────────────────────────────────────┘  │
│  ⚙ Settings   │                                      [Search]      │
│                │                                                    │
│                │  Searching again...                               │
│                │                                                    │
│                │  Narrowed by                                      │
│                │  [PDFs ×] [Documents ×] [Clear]                   │
│                │                                                    │
└────────────────────────────────────────────────────────────────────┘
```

Rules:

- search input is restored immediately;
- narrowing choices are restored immediately;
- results are refreshed;
- no stale snapshot is shown as current.

---

## 9.6. Search History Disabled

```text
┌────────────────────────────────────────────────────────────────────┐
│  Settings                                                          │
│                                                                    │
│  Privacy                                                           │
│                                                                    │
│  Documents are processed on this computer only.                    │
│                                                                    │
│  Remember recent searches                                          │
│  [Off]                                                             │
│  Recent searches are not saved.                                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

Rules:

- if disabled, no new recent search entries are stored;
- existing entries should be cleared after confirmation or explicit choice;
- strict privacy mode should force this off.

---

## 9.7. Clear Recent Searches Confirmation

For normal clearing, a confirmation is optional because history can be harmlessly removed. But because the action cannot be undone, a lightweight confirmation is recommended.

```text
┌──────────────────────────────────────────────────────────┐
│  Clear recent searches?                                  │
│                                                          │
│  This removes the list of searches shown in orbok.        │
│  Your files and search data are not deleted.              │
│                                                          │
│  [Cancel]  [Clear recent searches]                       │
└──────────────────────────────────────────────────────────┘
```

Rules:

- Cancel first;
- explain that source files are not deleted;
- do not say database/history table/cache.

---

## 10. Search History Entry Layout

## 10.1. Compact Entry

```text
authentication token rotation
PDFs · Documents · 10 minutes ago
```

## 10.2. Japanese Entry

```text
監査 証跡 ログ
All folders · yesterday
```

## 10.3. With Narrowing Choices

```text
invoice approval policy
PDFs · This month · Reports · 2 days ago
```

## 10.4. Entry Actions

Default:

```text
[Search again]
```

Optional in future:

```text
[Keep this search]
[Remove]
```

---

## 11. History Storage Policy

## 11.1. Maximum Count

Recommended default:

```text
20 recent searches
```

Advanced setting may allow:

```text
Off
10
20
50
```

But default Settings should not expose count unless needed.

## 11.2. Deduplication

If the user repeats the same search with the same narrowing choices:

- move the existing entry to the top;
- update timestamp;
- do not create duplicate entries.

If search words are the same but narrowing choices differ:

- keep separate entries;
- show different summaries.

## 11.3. Empty Search

Do not store empty searches.

## 11.4. Failed Search

Do not store failed searches unless results were previously valid and the user explicitly reopened one.

## 11.5. Search While Preparing

If the user searches while folders are still being prepared:

- history may store the search;
- reopening should search current prepared data;
- do not promise identical result counts.

---

## 12. State Model

## 12.1. SearchHistoryEntry

```rust
#[derive(Debug, Clone)]
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

## 12.2. StoredSearchFilter

```rust
#[derive(Debug, Clone)]
pub enum StoredSearchFilter {
    Folder {
        id: FolderId,
        label: String,
    },
    Kind {
        id: KindId,
        label: String,
    },
    Changed {
        value: ChangedFilter,
        label: String,
    },
    ReadyStatus {
        value: ReadyFilter,
        label: String,
    },
    SearchStyle {
        value: SearchStyle,
        label: String,
    },
    Language {
        value: LanguageFilter,
        label: String,
    },
}
```

## 12.3. SearchHistorySettings

```rust
#[derive(Debug, Clone)]
pub struct SearchHistorySettings {
    pub remember_recent_searches: bool,
    pub max_entries: usize,
    pub clear_when_privacy_strict: bool,
}
```

## 12.4. SearchUiState Additions

```rust
#[derive(Debug, Clone)]
pub struct SearchUiState {
    pub history: Vec<SearchHistoryEntry>,
    pub history_panel_open: bool,
    pub restoring_history_id: Option<SearchHistoryId>,
}
```

---

## 13. Message Model

```rust
#[derive(Debug, Clone)]
pub enum SearchHistoryMessage {
    OpenRecentSearches,
    CloseRecentSearches,
    SearchAgain(SearchHistoryId),
    RecentSearchRestored(SearchHistoryId),
    RemoveRecentSearch(SearchHistoryId),
    AskClearRecentSearches,
    CancelClearRecentSearches,
    ConfirmClearRecentSearches,
    ToggleRememberRecentSearches(bool),
}
```

Rules:

- SearchAgain restores state immediately;
- actual search runs after restore;
- if folder from history no longer exists, drop that filter and show a notice;
- if remembering is turned off, no new entries are stored.

---

## 14. Reopen Logic

When reopening a recent search:

```text
load entry
restore search text
restore filters that are still valid
drop filters that no longer apply
show notice if any were dropped
run search again
update history timestamp
```

Friendly notice:

```text
Some choices from this search are no longer available, so orbok searched the remaining choices.
```

Example:

A folder was removed after the search was saved.

```text
The folder “Reports” is no longer available. Showing all folders instead.
```

---

## 15. Privacy and Diagnostics

## 15.1. Privacy Mode

If strict privacy mode is enabled:

- turn off search history;
- clear existing history after asking the user;
- do not store future history.

Copy:

```text
Recent searches are not saved while strict privacy is on.
```

## 15.2. Diagnostics Export

Diagnostics export must not include search history by default.

If a support export ever includes search history, it must require explicit opt-in.

Copy:

```text
Include recent searches in the support file
```

Default:

```text
Off
```

## 15.3. Logs

Do not log search text by default.

If debug logging is enabled, search text must still be redacted unless the user explicitly enables detailed diagnostics.

---

## 16. Accessibility Requirements

- Recent search entries must be keyboard focusable.
- Each entry must expose a clear action such as Search again.
- Timestamps must be readable text.
- Clear recent searches must be reachable by keyboard.
- Confirmation dialog must focus Cancel first.
- Pressing Escape closes the history drawer.
- Reopening a search must not unexpectedly move focus away from the search input.
- Screen reader label should include search text and summary.

Example accessible label:

```text
Search again: authentication token rotation, PDFs, Documents, 10 minutes ago
```

---

## 17. Interaction Rules

## 17.1. Creating History Entry

Create or update a history entry after a successful search.

A successful search means:

- search request completed;
- result count is known;
- no fatal problem occurred.

Searches with zero results may be stored, because users may want to revisit them, but avoid highlighting them in the main recent list unless needed.

## 17.2. Clicking Recent Search

Clicking a recent search:

- restores search text;
- restores narrowing choices;
- closes the history panel;
- starts search again.

## 17.3. Removing One Entry

Optional P1 behavior:

```text
Remove from recent searches
```

No confirmation needed for one entry.

## 17.4. Clearing All Entries

Use lightweight confirmation.

## 17.5. Turning History Off

When user turns history off:

```text
Turn off recent searches?

orbok will stop saving searches. You can also clear searches already saved.

[Cancel] [Turn off] [Turn off and clear]
```

---

## 18. Implementation Priority

## 18.1. P0

Implement:

- store recent searches locally;
- restore search text and filters;
- search again against current files;
- compact Recent searches list;
- Clear recent searches;
- Remember recent searches setting;
- privacy copy;
- no result tabs by default.

## 18.2. P1

Implement:

- recent searches drawer;
- remove single entry;
- dropped-filter notice;
- strict privacy integration;
- diagnostics redaction test.

## 18.3. P2

Implement:

- Keep this search;
- kept searches list;
- optional result-count preview;
- history search;
- named saved searches.

## 18.4. Not Recommended for Initial Release

Do not implement by default:

- automatic search tabs;
- multiple simultaneous result workspaces;
- browser-style tab close/restore;
- pinboard of result sets;
- search workspace manager.

---

## 19. Acceptance Criteria

The design is accepted when:

1. Users can reopen recent searches.
2. Reopened searches run against current files.
3. Search text and narrowing choices are restored.
4. Invalid old folder filters are safely dropped with a friendly notice.
5. History is local-only.
6. User can clear recent searches.
7. User can turn search history off.
8. Strict privacy mode disables history.
9. Diagnostics do not include search history by default.
10. The default UI does not use result tabs.
11. The UI does not expose technical terms.
12. Keyboard access works for recent search entries.

---

## 20. Final Product Decision

orbok should support search history as:

```text
Recent searches
```

It should not use automatic search-result tabs by default.

Future saved searches should be intentional:

```text
Keep this search
```

This keeps the app simple for non-technical users while still allowing users to return to previous work.
