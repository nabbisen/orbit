# Implementation Handoff — RFC-043: Model Download Readiness Check and Bounded Concurrency

**Project:** orbok  
**RFC:** 043  
**Implementation theme:** local model-file readiness, bounded download concurrency, safe retry  
**Primary owners:** app setup/model manager/download worker/settings

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Implement robust model setup:

```text
Check local files first.
Download only what is missing or invalid.
Use max_concurrent = 2.
Validate before marking better search ready.
```

The user sees one friendly setup progress experience.

## 2. Scope

### In scope

- Local readiness check at startup, wizard initiation, retry, and locate-existing flow.
- Required files:
  - `tokenizer.json`
  - `onnx/model.onnx`
- Skip valid files.
- Download missing/invalid files only.
- `.part` temporary files.
- Final validation and atomic rename.
- Bounded concurrency of 2.
- Retry only failed/missing files.
- Rate-limit/backoff-friendly behavior.

### Out of scope

- Model selection policy.
- Automatic model updates.
- Multi-model queue.
- User-visible concurrency setting.
- HTTP resume as P0.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/app/src/setup_wizard.rs
crates/models/src/readiness.rs
crates/models/src/download_plan.rs
crates/models/src/validation.rs
crates/app/src/download_worker.rs
crates/ui/src/screens/setup_wizard.rs
```

## 4. Data / State Changes

Add statuses:

```rust
pub enum LocalFileStatus {
    Ready,
    Missing,
    Partial,
    Invalid,
    CannotCheck,
}
```

```rust
pub enum DownloadAction {
    Skip,
    Download,
    Replace,
    Retry,
}
```

```rust
pub struct DownloadPlan {
    pub model_id: ModelId,
    pub files: Vec<ModelFilePlan>,
    pub max_concurrent: usize,
}
```

## 5. PR Plan

### PR-034-1 — Readiness scanner

Tasks:

- Check required file existence, regular file, non-empty, readable.
- Add expected size/checksum hooks if manifest exists.
- Return readiness report.
- Run on startup and setup wizard entry.

Acceptance:

- Both valid files skip wizard or show ready.
- One missing file generates one download action.

### PR-034-2 — Download plan and `.part` policy

Tasks:

- Generate plan from readiness report.
- Use temp path for downloads.
- Validate temp file before final rename.
- Remove or restart stale partial files.

Acceptance:

- `.part` file is never treated as ready.
- Final file appears only after validation.

### PR-034-3 — Bounded concurrency

Tasks:

- Implement max 2 concurrent file downloads.
- Apply only to needed files.
- Combine progress into one UI progress state.

Acceptance:

- Two missing files can download concurrently.
- One ready / one missing downloads only missing.
- Default UI does not expose concurrency.

### PR-034-4 — Retry and failure handling

Tasks:

- Retry begins with local readiness check.
- Completed files are skipped.
- Handle network/server/rate-limit/disk/write failures with friendly errors.

Acceptance:

- Retry does not redownload valid files.
- Failed download does not mark model ready.

### PR-034-5 — UI wizard updates

Tasks:

- Add checking state.
- Add “download only what is needed” copy.
- Add ready state.
- Keep “Use basic search only.”

Acceptance:

- Basic search remains available if setup fails or is skipped.

## 6. UI Copy

```text
Checking search helper...
Better search is ready.
Some search helper files are needed.
orbok will download only what is missing.
Downloading better search
Your files stay on this computer.
Download did not finish.
Please check your connection and try again.
```

## 7. Acceptance Criteria

- Local files checked before download.
- Valid files skipped.
- Missing/invalid files downloaded.
- Partial files not treated as ready.
- Concurrency internally limited to 2.
- Model ready only after all required files pass validation.
- Retry re-checks local files.
- Default UI shows one friendly progress experience.
- Basic search remains available.

## 8. QA Checklist

- Fresh install: both files missing.
- Restart after tokenizer complete and model partial.
- Existing valid model folder.
- Existing empty model file.
- Failed network then retry.
- Rate-limit/server error simulation.
- Disk write error simulation where possible.
- Verify no ready state until both files valid.
