# RFC-045 Task Breakdown and PR Plan

**Project:** orbok  
**RFC:** 045  
**Document:** Task Breakdown / PR Plan  
**Date:** 2026-06-20  
**Status:** Accepted plan, self-reviewed  

---

## PR 1 — Search Location State and Data Types

### Goal

Introduce search location state without changing UI behavior yet.

### Tasks

- Add `SearchFolderScope`.
- Add `SearchLocationState`.
- Add `SearchLocation` or equivalent.
- Add selected search location to search UI/app state.
- Add helper for display labels:
  - `{folder} and subfolders`
  - `{folder} only`
- Add default scope = `FolderAndSubfolders`.

### Tests

- default state has no selected location;
- default scope is subfolders;
- display labels are correct;
- clearing location preserves query state.

### Acceptance

- App compiles.
- Existing search behavior remains unchanged.
- No user-visible regression.

---

## PR 2 — Folder Picker on Search Submit

### Goal

When Search is pressed without selected location, open folder picker.

### Tasks

- Modify submit-search command path.
- Launch the folder picker through a command/task, not from view rendering.
- Add `picker_in_progress` guard against duplicate dialogs.
- If no location is selected, call `rfd` folder picker.
- Preserve query while picker is open.
- Handle picker cancel as neutral state.
- Add `picker_in_progress` if needed to avoid duplicate dialogs.

### Tests

- submit without location requests picker;
- cancel picker keeps query;
- double-click Search does not open multiple pickers;
- folder picker result returns through message path;
- empty query behavior remains existing behavior.

### Acceptance

- First-run user can type query then receive folder picker on Search.

---

## PR 3 — Create or Reuse Remembered Folder

### Goal

After folder selection, create or reuse internal source/folder record.

### Tasks

- Normalize/canonicalize selected path as current source system expects.
- Check existing remembered folders.
- Reuse existing folder if path matches.
- Create new folder/source if not found.
- Set selected search location.
- Trigger source preparation job.
- Avoid duplicate folder records.

### Tests

- selected new folder creates one record;
- selecting same folder again reuses record;
- selecting existing folder does not duplicate;
- permission failure maps to friendly problem.

### Acceptance

- Folder picked from search appears as selected search location.
- Folders screen shows it as remembered/preparing.
- No duplicate records.

---

## PR 4 — Run Search After Folder Selection

### Goal

Continue the original search automatically after folder selection.

### Tasks

- Store pending query during folder picker.
- After folder record is ready, submit search.
- Search ready files immediately if any exist.
- Show preparation status if folder is still preparing.
- Update results as preparation progresses if existing architecture supports it.

### Tests

- query continues after folder selection;
- results appear when ready files exist;
- preparation status appears;
- cancel does not submit search.

### Acceptance

- User does not need to press Search twice after choosing a folder.

---

## PR 5 — Search Location UI

### Goal

Add visible, friendly search-location selector.

### Tasks

- Add `Search in: [Choose a folder]`.
- Add selected folder chip.
- Add `Change` action.
- Add clear/remove chip action.
- Add scope selector:
  - This folder and subfolders
  - This folder only
- Ensure keyboard accessibility.

### Tests

- choose folder button works;
- chip clear works;
- scope switch updates state;
- scope switch does not create a duplicate remembered folder/source;
- labels avoid “source” and “recursive.”

### Acceptance

- Search screen clearly shows where search will look.

---

## PR 6 — Recent / Remembered Folder Shortcuts

### Goal

Let users quickly reuse remembered folders.

### Tasks

- Query remembered folders for recent or common folder chips.
- Show small row:
  - Recent folders: Documents, Downloads, ...
- Selecting chip sets search location.
- Respect strict privacy if recent folder persistence is disabled.
- Use remembered folder list even if recent tracking is off.

### Tests

- recent folder chip selects location;
- strict privacy suppresses recent folder history if required;
- missing folder chip shows recovery or disabled state.

### Acceptance

- Repeat searches require fewer clicks.

---

## PR 7 — Folders Screen Copy Alignment

### Goal

Ensure folder management remains secondary and friendly.

### Tasks

- Replace default “source” labels in affected UI with “folder.”
- Add copy:
  - Remove from orbok
  - Your files will not be deleted
- Ensure search screen does not route first-run users to Folders first.

### Tests

- no “source” label in default search flow;
- remove folder copy is safe;
- Folders screen still supports preparation and recovery.

### Acceptance

- Users understand folder management without technical source language.

---

## PR 8 — End-to-End QA and Polish

### Goal

Finalize integrated behavior.

### Tasks

- Add E2E/manual QA script.
- Test first-run flow.
- Test picker cancel.
- Test existing folder reuse.
- Test missing folder.
- Test recursive/folder-only behavior.
- Test keyboard navigation.
- Confirm no source jargon in default flow.

### Acceptance

- RFC acceptance criteria pass.
- QA checklist passes.
