# HANDOFF RFC-045: Search-in-Folder Flow and Friendly Folder Management

**Project:** orbok  
**RFC:** 045  
**Handoff type:** Implementation Handoff  
**Target audience:** app developers, UI developers, state-management developers, QA  
**Date:** 2026-06-20  
**Status:** Accepted handoff, self-reviewed  

---

## 1. Implementation Outcome

After this work, a user must be able to:

```text
type search words
press Search
choose a folder only if no location is selected
see search begin automatically
```

The user must not need to visit the Folders screen before first search.

Internally, the app may still create/reuse a source/folder record. Externally, the UI must use the language:

```text
folder
search in
choose a folder
this folder and subfolders
```

not:

```text
source
register source
recursive
index source
```

---

## 2. Scope

Implement:

- search location state;
- folder picker on search submit when location is missing;
- remembered folder creation/reuse from selected folder;
- default `FolderAndSubfolders` scope;
- option to switch to `FolderOnly`;
- search resumes automatically after folder selection;
- recent/remembered folder chips if existing data is available;
- Folders screen remains secondary;
- copy changes from “source” to “folder” in this flow;
- tests and QA for first-run search.

---

## 3. Non-Scope

Do not implement in this PR set unless explicitly scheduled:

- full transient “Just this time” mode;
- OS file-manager context menu integration;
- dynamic live file watching;
- drag-and-drop folder search;
- new search ranking;
- new extraction pipeline;
- new design system styling from snora team.

---

## 4. Module Impact

Likely touched areas:

```text
orbok-ui
  search view
  folder/location selector
  empty state
  folder picker orchestration

orbok-app / app state
  search state
  selected search location
  submit-search flow

orbok-source / source management
  create/reuse folder record
  source path normalization
  duplicate detection

orbok-scheduler
  user-visible preparation jobs

orbok-db
  source/folder record persistence
  recent folder metadata if implemented

settings/privacy
  recent folder behavior in strict mode
```

Exact crate/module names may differ. Preserve existing architecture.

---

## 5. UI Flow Requirements

### 5.1. First Run

Initial screen:

```text
Search

[ What are you looking for?                         ]

Search in: [Choose a folder]

[Search]

You can choose a folder when you search.
Your files stay on this computer.
```

### 5.2. Submit Without Folder

When user clicks Search:

1. preserve query text;
2. open folder picker;
3. wait for user selection;
4. if selected, create/reuse folder;
5. set selected search location;
6. start preparation;
7. start search.

### 5.3. Cancel Folder Picker

If user cancels:

- keep query;
- keep screen stable;
- do not show error;
- optionally show neutral helper:

```text
Choose a folder to search.
```

### 5.4. Selected Folder

Show:

```text
Search in: [Documents and subfolders ×] [Change]
```

The chip must be keyboard removable.

---

## 6. Source/Folder Creation Rule

When a folder is selected:

```text
normalize path
check if already remembered
if exists: reuse source_id
if not: create source/folder record
set current search location
```

Do not create duplicates for the same canonical folder.

---

## 7. Search Scope Rule

Default:

```rust
SearchFolderScope::FolderAndSubfolders
```

User-facing:

```text
This folder and subfolders
```

Alternative:

```rust
SearchFolderScope::FolderOnly
```

User-facing:

```text
This folder only
```

The scope must be stored with the selected search location. It must not create a second remembered folder/source record for the same root path; it is a search-time restriction in P0.

---

## 8. Progressive Preparation Rule

Do not block search until the whole folder is prepared.

Required behavior:

```text
start search against ready files
show preparation status
update results as more files become ready
```

Copy:

```text
Some files are still being prepared.
Results will improve as preparation finishes.
```

---

## 9. State Additions

Add or adapt:

```rust
pub struct SearchLocationState {
    pub selected: Option<SearchLocation>,
    pub recent_locations: Vec<SearchLocationSummary>,
    pub picker_in_progress: bool,
}
```

```rust
pub enum SearchLocation {
    Remembered {
        source_id: SourceId,
        display_name: String,
        scope: SearchFolderScope,
    },
    Transient {
        path: PathBuf,
        display_name: String,
        scope: SearchFolderScope,
    },
}
```

For this implementation, `Transient` can be omitted or reserved.

---

## 10. Messages / Commands

Add message cases similar to:

```rust
pub enum SearchLocationMessage {
    ChooseFolderRequested,
    FolderPickerCancelled,
    FolderPicked(PathBuf),
    SearchLocationSelected(SearchLocation),
    SearchLocationCleared,
    SearchScopeChanged(SearchFolderScope),
    RecentFolderSelected(SourceId),
}
```

Search submit should become:

```text
if query empty:
    follow existing empty-query behavior
else if no selected search location:
    open folder picker
else:
    run search
```

---


## 11.0. Non-Blocking Folder Picker Rule

Do not open the `rfd` folder picker directly from view rendering.

Use the normal iced command/task path:

```text
SubmitSearch
  ↓
ChooseFolderRequested
  ↓
folder picker async task
  ↓
FolderPicked(path) or FolderPickerCancelled
```

Set `picker_in_progress = true` while the dialog is active so repeated Search clicks do not open multiple dialogs.

## 11. Error Handling

| Situation | Behavior |
|---|---|
| picker cancelled | no error |
| selected folder unreadable | show friendly permission message |
| selected folder already exists | reuse |
| selected folder missing after selection | show folder-not-found recovery |
| source creation fails | show friendly “could not save folder” message |
| preparation fails for some files | show partial-preparation message |

---

## 12. Copy Requirements

Required default copy:

```text
Search in
Choose a folder
This folder and subfolders
This folder only
Searching in {folder}
Preparing files in the background
Your files stay on this computer
```

Forbidden default copy:

```text
source
register source
recursive
index
reindex
worker
queue
```

---

## 13. Acceptance Gate

The implementation is not complete until a first-run user can search without visiting Folders.

Manual happy path:

```text
open fresh app
type "invoice"
click Search
choose Documents
see search start
see folder preparation status
```

---

## 14. Developer Notes

This is primarily an orchestration and UX-state change.

Avoid overengineering transient folder mode in P0. The stable path is:

```text
folder chosen from search → remembered folder/source record
```

Add “Just this time” later only if folder-list clutter becomes a real UX problem.
