# Implementation Handoff — RFC-037: Source Lifecycle, Refresh Policy, and Change Detection UX

**Project:** orbok  
**RFC:** 037  
**Implementation theme:** registered folder lifecycle, refresh, and missing-folder recovery  
**Primary owners:** fs/scanner/source manager/ui/workers

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement stable folder lifecycle behavior:

```text
startup check + manual refresh first
live watcher deferred
missing folders recoverable
changed files marked needs update
source files never deleted
```

## 2. Scope

### In scope

- Source states.
- File states.
- Startup check.
- Manual refresh.
- Missing folder recovery.
- Change detection by metadata.
- External drive/network folder gentle behavior.
- Symlink safety.
- Change storm summary.

### Out of scope

- Live file watcher as required behavior.
- Cloud folder sync.
- Remote drive protocol support.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/fs/src/source_lifecycle.rs
crates/fs/src/scanner.rs
crates/fs/src/fingerprint.rs
crates/workers/src/source_refresh.rs
crates/ui/src/screens/folders.rs
crates/data/catalog/source_repo.rs
crates/data/catalog/file_repo.rs
```

## 4. Data / State Changes

Add source state:

```rust
pub enum SourceState {
    Active,
    Preparing,
    NeedsUpdate,
    Paused,
    FolderNotFound,
    PermissionProblem,
    Removed,
}
```

Add file state:

```rust
pub enum FileState {
    Discovered,
    Preparing,
    Ready,
    NeedsUpdate,
    PartlyPrepared,
    CouldNotPrepare,
    FileNotFound,
    Ignored,
}
```

## 5. PR Plan

### PR-037-1 — Source and file state model

Tasks:

- Add states to catalog if not present.
- Add transition helpers.
- Add user-facing label mapping.

Acceptance:

- All source/file states are representable.

### PR-037-2 — Startup check

Tasks:

- Check registered folders on startup.
- Mark folder missing/permission problem.
- Mark obvious changed/missing files.
- Queue refresh jobs via scheduler.

Acceptance:

- App starts with honest folder states.

### PR-037-3 — Manual refresh

Tasks:

- Add Check again / Prepare again actions.
- Recheck folder existence and metadata.
- Queue bounded refresh.

Acceptance:

- Missing folder can recover after reconnect.

### PR-037-4 — Missing folder UI

Tasks:

- Show Folder not found card.
- Actions: Check again, Choose folder again, Remove from orbok.
- Copy explains files are not deleted.

Acceptance:

- User understands external drive scenario.

### PR-037-5 — Ignore/symlink/change storm safeguards

Tasks:

- Apply conservative ignore defaults or document none.
- Prevent symlink escape outside chosen folder.
- Batch many changes into one summary.

Acceptance:

- Change storm does not create noisy UI.

## 6. UI Copy

```text
Ready
Preparing
Needs update
Folder not found
Cannot open
Check again
Choose folder again
Remove from orbok
Your files were not deleted. orbok just cannot find this folder right now.
```

## 7. Acceptance Criteria

- Startup check exists.
- Manual refresh exists.
- Missing folders are recoverable.
- Removed folders do not delete files.
- Changed files become Needs update.
- Prepared data remains searchable during refresh.
- Live watcher is not required.
- Default UI avoids source/index/reindex terms.

## 8. QA Checklist

- Add folder.
- Edit file and refresh.
- Delete file and refresh.
- Rename source folder.
- Disconnect simulated external source.
- Reconnect and Check again.
- Remove from orbok and confirm files remain.
- Test symlink outside chosen folder.
- Test many file changes.
