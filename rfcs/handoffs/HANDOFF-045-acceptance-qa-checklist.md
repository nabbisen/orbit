# RFC-045 Acceptance and QA Checklist

**Project:** orbok  
**RFC:** 045  
**Document:** Acceptance / QA Checklist  
**Date:** 2026-06-20  
**Status:** Accepted checklist, self-reviewed  

---

## 1. Acceptance Criteria

The implementation is accepted when all items below pass.

- [ ] User can type a query before choosing a folder.
- [ ] Pressing Search without selected location opens a folder picker.
- [ ] Cancelling folder picker keeps the query.
- [ ] Cancelling folder picker does not show an error.
- [ ] Selecting a folder starts search automatically.
- [ ] Selected folder is created as a remembered folder in P0.
- [ ] Existing remembered folder is reused, not duplicated.
- [ ] Changing search scope does not create a duplicate remembered folder.
- [ ] Default scope is “This folder and subfolders.”
- [ ] User can switch to “This folder only.”
- [ ] Search results can appear before full folder preparation completes.
- [ ] Search screen shows selected location clearly.
- [ ] Folders screen is not required before first search.
- [ ] UI says “folder,” not “source,” in default flow.
- [ ] User can remove remembered folder from orbok without deleting files.

---

## 2. Manual QA — First-Run Happy Path

1. Start orbok with no folders configured.
2. Type a query, e.g. `invoice`.
3. Click Search.
4. Confirm folder picker opens.
5. Select a folder.
6. Confirm search starts automatically.
7. Confirm preparation status appears.
8. Confirm UI says:
   - `Searching in {folder}`
   - `Preparing files in the background`
9. Confirm Folders screen was not required.

Expected result:

```text
User reaches search results or preparing-results state in one natural flow.
```

---

## 3. Manual QA — Picker Cancel

1. Start with no selected folder.
2. Type query.
3. Click Search.
4. Cancel folder picker.

Expected:

- query remains;
- no red error;
- user can click Search again;
- helper text may say `Choose a folder to search.`

---

## 4. Manual QA — Existing Folder Reuse

1. Add or remember `Documents`.
2. Clear selected search location.
3. Type query.
4. Click Search.
5. Select `Documents` again.

Expected:

- existing folder/source record reused;
- no duplicate in Folders screen;
- search starts.

---

## 5. Manual QA — Scope Control

1. Select folder.
2. Confirm chip says `{folder} and subfolders`.
3. Change scope to `This folder only`.
4. Run search.
5. Confirm state persists for current search location.

Expected:

- scope label is clear;
- no term `recursive` appears in default UI.

---

## 6. Manual QA — Preparation Still Running

1. Select a folder with many files.
2. Search immediately.

Expected:

- search does not wait for all files;
- status says some files are still being prepared;
- results update if existing architecture supports live update;
- app remains responsive.

---

## 7. Manual QA — Missing Folder

1. Remember a folder on external/removable location.
2. Remove/disconnect it.
3. Try to select/search it.

Expected:

- UI shows `Folder not found`;
- recovery actions appear:
  - Check again
  - Choose folder again
  - Remove from orbok
- user files are not implied to be deleted.

---

## 8. Manual QA — Remove From orbok

1. Remember a folder.
2. Remove it from orbok.

Expected copy:

```text
Remove from orbok
Your files will not be deleted.
```

Expected:

- folder no longer appears as remembered;
- source files remain untouched.

---

## 9. Accessibility QA

- [ ] Search input is keyboard reachable.
- [ ] Choose folder button is keyboard reachable.
- [ ] Folder chip clear action is keyboard reachable.
- [ ] Scope selector is keyboard reachable.
- [ ] Focus returns to useful place after picker closes.
- [ ] Repeated Search clicks while picker is open do not open multiple dialogs.
- [ ] Status is text-visible, not color-only.
- [ ] Search location label is readable by assistive tooling where supported.

---

## 10. Copy QA

Forbidden default terms:

- [ ] `source`
- [ ] `register source`
- [ ] `recursive`
- [ ] `index`
- [ ] `reindex`
- [ ] `worker`
- [ ] `queue`

Required terms:

- [ ] `folder`
- [ ] `Search in`
- [ ] `Choose a folder`
- [ ] `This folder and subfolders`
- [ ] `This folder only`
- [ ] `Your files stay on this computer`

---

## 11. Regression QA

- [ ] Existing remembered folders still work.
- [ ] Existing search history still works.
- [ ] Narrow results still works.
- [ ] Folder preparation status still works.
- [ ] Privacy strict mode does not save recent folders if configured.
- [ ] Diagnostics are unaffected.
- [ ] Model download flow is unaffected.
- [ ] Basic keyword search works without AI model.

---

## 12. Done Definition

RFC-045 is done when:

```text
A new user can search naturally without learning source management,
and an existing user can still manage remembered folders safely.
```
