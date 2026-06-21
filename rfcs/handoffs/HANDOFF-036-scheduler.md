# Implementation Handoff — RFC-036: Resource-Aware Indexing Scheduler and Backpressure

**Project:** orbok  
**RFC:** 036  
**Implementation theme:** weak-machine-friendly indexing and background work control  
**Primary owners:** workers/scheduler/pipeline/search UI

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Make orbok feel calm and responsive while folders are being prepared.

The scheduler must:

```text
prioritize search and UI
bound background queues
pause/resume safely
recover after restart
avoid unbounded CPU/memory pressure
```

## 2. Scope

### In scope

- Work queues for scan/extract/chunk/keyword/embedding/maintenance.
- Priority levels.
- Conservative default worker counts.
- Backpressure.
- Pause/resume/cancel.
- User activity mode.
- Crash/restart recovery hooks.
- UI progress copy.

### Out of scope

- Full battery/thermal detection as P0.
- GPU scheduling.
- Advanced user tuning UI.
- File watcher details; see RFC-037.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/workers/src/scheduler.rs
crates/workers/src/queue.rs
crates/workers/src/job.rs
crates/workers/src/recovery.rs
crates/app/src/app_events.rs
crates/ui/src/screens/indexing.rs
crates/data/catalog/jobs_repo.rs
```

## 4. State / Data Changes

Add:

```rust
pub enum WorkPriority {
    UserBlocking,
    UserVisible,
    NormalBackground,
    LowBackground,
    Maintenance,
}
```

```rust
pub enum JobState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    WaitingForDependency,
}
```

```rust
pub enum ResourceMode {
    Normal,
    UserActive,
    LowImpact,
    Paused,
}
```

## 5. PR Plan

### PR-036-1 — Job model and persistence

Tasks:

- Define job kinds and states.
- Persist job state and attempt count.
- Add startup recovery transition rules.

Acceptance:

- Interrupted Running jobs recover safely.

### PR-036-2 — Bounded queue implementation

Tasks:

- Add separate bounded queues.
- Enforce capacities.
- Add backpressure events.

Acceptance:

- Large scan cannot create unbounded memory growth.

### PR-036-3 — Priority and user activity policy

Tasks:

- Add priority dispatch.
- Detect active search/typing.
- Reduce background work during user activity.
- Ensure embedding yields to search.

Acceptance:

- Search does not wait behind embedding work.

### PR-036-4 — Pause/resume/cancel

Tasks:

- Add pause/resume actions.
- Cancel jobs when folder removed.
- UI copy: Pause preparing / Resume preparing.

Acceptance:

- User can pause preparation and still search ready files.

### PR-036-5 — UI progress and partial readiness

Tasks:

- Show ready count and preparing state.
- Show “You can search now” when partial data exists.
- Avoid technical queue labels.

Acceptance:

- UI remains understandable and non-technical.

## 6. Acceptance Criteria

- Background work uses bounded queues.
- Search/UI has priority.
- Embedding yields to active search.
- Pause/resume works.
- Removing folder cancels queued work.
- Restart recovery works.
- Partial readiness is visible.
- No technical scheduling terms in default UI.

## 7. QA Checklist

- Add folder with many files.
- Search during preparation.
- Pause and resume preparation.
- Remove folder during preparation.
- Simulate extractor failure.
- Simulate app restart during jobs.
- Verify embedding does not freeze search.
- Verify memory does not grow unbounded with large scan.
