# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.2.0] — 2026-06-07

### Added

**M5 — Adaptive Chunking (RFC-006)**
- `orbok-extract` chunker module: structure-aware chunking for Markdown
  (one child chunk per heading section) and paragraph-based fallback for
  plain text, with overlapping windows for long sections.
- Parent-child chunk model: document-level parent chunk (ordinal 0) plus
  leaf chunks used for retrieval.
- Explicit location quality per chunk: `exact` for text/Markdown line
  ranges, `approximate` for fallback windows.
- Chunk content hash (SHA-256 of normalized text) for stale detection.

**M6 complete — Keyword Search Pipeline (RFC-007)**
- `orbok-workers` crate: synchronous `ExtractionWorker`, `ChunkAndIndexWorker`,
  and `run_pending` coordinator.
- **Replace-on-success** transaction in `ChunkRepository::insert_bundle`:
  new chunks and FTS rows committed atomically; previous active index
  survives any failure.
- `SearchService`: keyword search returning `Vec<SearchResult>` with
  dynamic snippet loading from source files (FR-091).
- `SnippetLoader`: reads stored line ranges from source files; returns
  `None` when source is unavailable without crashing.
- `SearchService::search` available for use by `orbok-app`.

**M9 partial — Search Result Display**
- `SearchResultDisplay` view-model struct in `orbok-ui`.
- Search view renders result cards: title, display path, heading context,
  dynamic snippet, and badge list.
- Running/no-results/results-ready states in the search view.

**RFC housekeeping**
- RFCs 001–007, 027, 031 moved to `rfcs/done/`.
- `rfcs/README.md` index rebuilt to reflect current state.

### Changed
- `AppState` gains `search_results: Vec<SearchResultDisplay>` and
  `search_running: bool`; `Message` gains `SearchResultsReady` and
  `SearchError` variants.
- `FileRepository` gains `get_by_id(file_id)`.
- `orbok-fs` now exports `GuardedSource`.
- `orbok-db/repo` now re-exports `ExtractionId`, `JobStatus`, `JobType`
  from `orbok-core` as convenience aliases.
- Baseline migration updated pre-release: `chunk_fts` drops `chunk_id`
  and `file_id` UNINDEXED columns (contentless tables store no values);
  `keyword_index_records` gains `fts_rowid INTEGER` for the chunk ↔ FTS
  row mapping.

### Tests
- `orbok-extract`: 15 tests (adds 6 RFC-006 chunker tests).
- `orbok-workers`: 5 integration tests covering the full
  extract → chunk → index → search pipeline, including snippet loading
  and rechunk-failure preservation.
- Workspace total: **88 tests / 0 failures**.

---

## [0.1.0] — 2026-06-07

### Added

**Foundation (M0–M1)**
- Rust 2024 edition Cargo workspace with nine crates.
- RFC-001: three-layer data lifecycle (persistent / rebuildable / ephemeral).
- RFC-002: SQLite catalog schema with append-only migrations, FTS5
  contentless keyword index, foreign-key enforcement.

**Source boundary (M2)**
- RFC-003: source registration, canonical path enforcement, symlink
  policy, hidden-file policy, sensitive-directory warnings.

**File scanner (M3)**
- RFC-004: recursive directory walker, nanosecond-precision mtime
  comparison, SHA-256 content hashing, stale/missing/discovered state
  machine, cancellation support, index-job queueing.

**Extraction (M4)**
- RFC-005: extractor trait, plain-text and Markdown extractors with
  line-aware offsets, normalization pipeline, extractor version tracking.

**Cache engine (Appendix A)**
- localcache 0.20.0 integration: `MetadataThenFullHash` change detection,
  namespace policy, plan-validated cleanup.

**Keyword search (M6 prototype)**
- RFC-007: FTS5 contentless engine behind `KeywordSearchEngine` trait;
  safe query building (RFC-015 injection neutralization).

**GUI and i18n (RFC-027, RFC-031)**
- snora 0.8 / iced 0.14 application shell with six-page sidebar.
- Typed i18n catalog: English and Japanese, exhaustive at compile time.
- Headless `--check` mode for CI / display-less environments.

### Dependencies (pinned)
- localcache 0.20.0 (mtime nanosecond precision, schema v5).
- rusqlite 0.40 (single libsqlite3-sys instance shared with localcache).
- iced 0.14 via snora 0.8.

---

## [0.3.0] — 2026-06-07

### Added

**M7 — Embedding and Vector Search (RFC-008)**
- `EmbeddingModel` trait in `orbok-models` (RFC-008 §6): `embed_batch`,
  `name`, `version`, `dimension`. Implementations must run locally and
  never transmit text externally.
- `MockEmbeddingModel`: 8-dimensional deterministic mock using SHA-256
  as a pseudo-random source; L2-normalized output. Used for pipeline
  testing without a real ML runtime.
- Vector serialization helpers: `vec_to_blob`/`blob_to_vec` (FP32
  little-endian, RFC-008 §12.1).
- `VectorCandidate` type; cosine-similarity and L2-normalize utilities.
- `EmbeddingId` added to orbok-core.
- `EmbeddingRepository` in orbok-db: `upsert`, `list_active_for_scan`
  (joins with chunks to exclude stale chunks), `mark_stale_for_model`,
  `count_active`.
- `EmbeddingWorker` in orbok-workers: reads extraction cache → embeds
  chunk texts in batch → stores vectors. `with_mock` constructor for
  tests and no-model operation.
- `ExactVectorSearch`: cosine-similarity scan over all active embeddings
  for a model (RFC-008 §13 "exact search first").

**M8 — Hybrid Search and RRF (RFC-009)**
- `rrf_fuse`: Reciprocal Rank Fusion (k=60), deduplicating by chunk_id,
  producing `FusedCandidate` with per-source rank metadata (RFC-009 §7).
- `HybridSearchService`: `keyword_only` and `with_model` constructors;
  `search(query, mode, limit)` running keyword + vector retrieval,
  RRF fusion, and snippet loading in one call (RFC-009 §12).
- `SearchMode` enum (RFC-009 §8): `Auto`, `Exact`, `Conceptual`, `Fast`
  with per-mode candidate limits.
- Badge system: `MatchBadge::Keyword`, `Semantic`; fused results carry
  both badges when both retrievers contributed.
- `SearchMode` in `orbok-ui` `AppState`; `SetSearchMode` message.

**i18n additions (RFC-031)**
- New keys: `SearchModeLabel`, `SearchModeAuto`, `SearchModeExact`,
  `SearchModeConceptual`, `SearchModeFast`, `BadgeKeyword`,
  `BadgeSemantic`, `BadgeFused` — translated to English and Japanese.
- `search_result_count(locale, n)` parameterized message.

### Tests
- `orbok-models`: 5 tests (adds embedding/vector ops tests).
- `orbok-workers`: 12 tests (adds 7 RFC-008/009 integration tests:
  embedding generation, vector search, RRF fusion, model-change
  staling, stale-chunk exclusion, catalog isolation).
- Workspace total: **99 tests / 0 failures**.

---

## [0.4.0] — 2026-06-07

### Added

**RFC-010 — Optional Local Reranking**
- `CrossEncoderReranker` trait and `RerankCandidate`/`RerankScore` types
  in `orbok-models`.
- `MockReranker`: deterministic mock ordering by passage length (test-safe,
  no ML runtime required).
- `HybridSearchService::with_reranker()` builder: attaches optional
  reranker that reorders the top-N fused results using passage text.
- `Fast` search mode bypasses reranking (`Limits.rerank = false`).
- Search remains fully functional with no reranker attached (RFC-010 §20).

**RFC-011 — Storage Dashboard**
- `update_storage_accounting(catalog, cache_db_path)` in orbok-workers:
  measures actual storage by category (keyword index rows, embedding BLOB
  sum, snippet cache bytes, localcache DB file size, event log rows).
- `StorageDataReady` message and `storage_rows` field in orbok-ui `AppState`.
- Storage view renders per-category breakdown with MiB values.
- `orbok-app` exposes `persist_locale()` helper — locale changes are now
  persisted to the catalog `app_settings` table.

**RFC-013 — Search View and Result Explanation UX**
- `SelectResult(usize)` message and `selected_result: Option<usize>` in
  `AppState`; result cards are now buttons that trigger selection.
- `OpenSourceFile(String)` message (canonical path) dispatched to orbok-app.
- `StorageDataReady` message wires real storage data into Storage view.
- Search mode selector row in the Search view (Auto / Exact / Conceptual).
- `search_result_count(locale, n)` parameterized i18n message.

**RFC-014 — Japanese and Mixed-Language Search**
- Migration 0002 (`0002_trigram_index.sql`): adds `chunk_fts_trigram`
  virtual table (FTS5 trigram tokenizer, SQLite 3.53.2) and
  `keyword_index_records.trigram_fts_rowid` column.
- `ChunkRepository::insert_bundle` now indexes every chunk in both
  the unicode61 and trigram FTS tables atomically.
- `MultilingualKeywordEngine`: detects CJK characters in the query
  (hiragana, katakana, CJK unified ideographs); routes CJK queries
  through both unicode61 and trigram tables, merging and deduplicating
  results. English/identifier queries use only unicode61.
- `normalize_query()`: converts fullwidth ASCII/digits (ＡＢＣ→ABC)
  and trims whitespace — satisfies RFC-014 §10 test 1.
- `contains_cjk()`: character-class-based CJK detector.
- `HybridSearchService` now uses `MultilingualKeywordEngine` internally
  for all keyword retrieval.

**Other improvements**
- Locale persistence: `PersistLocale` message variant; orbok-app
  `persist_locale()` writes to catalog settings on locale change.
- `orbok-ui` i18n: added keys `SearchModeLabel`, `SearchModeAuto`,
  `SearchModeExact`, `SearchModeConceptual`, `SearchModeFast`,
  `BadgeKeyword`, `BadgeSemantic`, `BadgeFused`, plus parameterized
  `search_result_count` in English and Japanese.

### Tests
- `orbok-models`: 7 tests (+2 reranker tests).
- `orbok-workers`: 26 tests (+14 covering RFC-010/011/013/014).
- Workspace total: **110 tests / 0 failures**.

---

## [0.5.0] — 2026-06-07

### Added

**RFC-012 — Model Registry and Installation Workflow (M12)**
- `ModelRepository` in orbok-db: full CRUD over the `models` catalog table
  with `insert`, `get`, `list_by_role`, `list_all`, `set_status`,
  `validate` (file-existence + dimension check), `locate` (register
  existing on-disk model), and `mark_embedding_dependents_stale`.
- `ModelRole` and `ModelStatus` enums with catalog-string round-trips.
- `ModelId` typed ID added to orbok-core.
- App works in keyword-only mode with empty model registry (RFC-012 §17).
- No model download occurs without explicit user action.

**RFC-015 — Security Hardening**
- `html_escape(raw)` in `orbok-search::snippet`: escapes `<>&"'` in
  snippet text before passing to the UI (RFC-015 §18 defense-in-depth).
- Security test module documents and exercises existing protections:
  PathGuard outside-source rejection, path-traversal via `..`, symlink
  escape blocking (all implemented in RFC-003/004, now explicitly
  labelled as security tests per RFC-015 §19).

**RFC-016 — Benchmark and Retrieval Evaluation Harness**
- New `orbok-bench` crate:
  - `corpus::generate(dir, n)` — synthetic Markdown documents (8
    templates: auth, storage, search, API, security, Japanese, code,
    models).
  - `queries::LABELED_QUERIES` — 8 labeled queries with expected
    document patterns.
  - `metrics::measure_search_latency` — p50/p95/p99 ms measurement
    with 3 warm-up rounds.
  - `metrics::compute_recall` — recall@5 against labeled queries.
  - `report::BenchmarkResult::write_json/write_markdown` — machine-
    readable and human-readable output (RFC-016 §12).
- Benchmark smoke test verifies the harness runs on a 10-document
  corpus without errors.

**RFC-017 — Packaging and Distribution**
- `--version` / `-V` flag in the orbok binary.
- `build.rs` in orbok-app embeds `CARGO_PKG_VERSION`.
- `scripts/checksum.sh` generates SHA-256 checksums for release archives.

**RFC-018 — Crash Recovery and Diagnostics**
- `run_startup_recovery(catalog, cache_path)` in orbok-workers:
  - Resets `running` → `queued` for jobs left by a crashed session.
  - Returns `RecoveryReport` with counts of reset and pending jobs.
  - Detects missing or corrupt cache DB (backup + recreate path).
- `check_catalog_integrity(catalog)` → `IntegrityReport`: detects
  orphaned child chunks, orphaned keyword/embedding records, and files
  without a parent source. Read-only; does not repair.
- `RecoveryReport` and `IntegrityReport` are printed at startup if
  anomalies are found.

**orbok-ui**
- `StorageDataReady` message and `storage_rows` field already wired
  in v0.4; `update_storage_accounting` now called after each pipeline
  run to keep storage view current.

### Tests
- `orbok-db`: 15 tests (model repo tested via v05 integration suite).
- `orbok-workers`: 37 tests (+11 covering RFC-012/015/016/018).
- Workspace total: **115 tests / 0 failures**.

---

## [0.6.0] — 2026-06-07 🎉 All Part 1–4 RFCs complete

This release completes the planned feature set defined in the initial
requirements document. All 23 implementation RFCs (RFC-000 through
RFC-020, RFC-027, RFC-031) are now in `rfcs/done/`.

### Added

**M10 complete — CleanupService end-to-end**
- `CleanupService` in orbok-workers: combines catalog-side cleanup
  (via `CleanupExecutor`) with cache-side cleanup (via `CacheService`)
  in one validated operation driven by `CleanupPlan`.
- `run_safe(plan)` — ordinary cleanup (snippet cache, search cache,
  stale indexes); guaranteed to never touch persistent source settings.
- `run_reset(plan, keep_settings)` — full catalog reset that also
  purges all localcache namespaces.
- `FullCleanupOutcome` reports `catalog_rows_deleted` and
  `cache_bytes_freed`.

**M12 backend infrastructure**
- `InferenceBackend` enum: `CandleCpu`, `CandleCuda`, `OnnxRuntime`, `Mock`.
- `EmbeddingModelConfig`: weights path, tokenizer path, dimension,
  max sequence length, backend, name/version.
- `RerankerConfig`: equivalent config for cross-encoder rerankers.
- `weights_exist()` validator on `EmbeddingModelConfig`.
- These types are consumed by the future candle/ONNX integration crates
  (RFC-021 implementation); the `MockEmbeddingModel` remains the
  fallback until a real backend is compiled in.

**RFC-019 — Test Matrix and Release Readiness**
- `.github/workflows/ci.yml`: four CI jobs:
  - **fast** (every PR): fmt, clippy, unit tests on non-GUI crates
  - **release** (main branch): release build, `--version`, `--check`, bench smoke
  - **security** (every PR): `cargo audit`, security test execution
  - **cross** (3 platforms): Linux, Windows, macOS smoke build
- `docs/src/maintainers/release_readiness.md`: release readiness levels
  RL-0 through RL-4, CI gate definitions, manual QA checklist,
  retrieval benchmark requirements, packaging checklist.

**RFC-020 — Documentation and User Guidance Structure**
Complete mdbook documentation covering all three user personas:
- **New users**: Features, Quick Start, Sources and Indexing, Searching,
  Storage and Cleanup, Local AI Models, FAQ
- **Intermediate users**: Settings Reference, Supported File Types
- **Maintainers**: Architecture Overview, Local Development, Testing
  Guide, RFC Index, Release Readiness

### Changed
- `rfcs/README.md`: all Part 1–4 RFCs now in `done/`; 0 in `proposed/`.
  RFC-021–030 remain in `draft/` as deferred future work.

### Tests
- `orbok-workers`: 46 tests (+9 covering M10/M12/RFC-019).
- Workspace total: **124 tests / 0 failures**.

---

## [0.7.0] — 2026-06-07

> **Note:** v1.0.0 is not yet confirmed. This release advances the
> pre-1.0 roadmap. See `ROADMAP.md` for v1.0.0 criteria.

### Added

**RFC-021 — Default Embedding Model Selection**
- New `orbok-embed` crate with the embedding backend factory:
  `create_embedding_model(config)` dispatches by `InferenceBackend`.
- `Mock` backend (always compiled): deterministic 8-dim vectors,
  no model files required — used in all tests.
- `OnnxRuntime` backend (`--features tract`): loads `.onnx` model via
  the pure-Rust `tract-onnx` runtime; `tract_backend.rs` is only
  compiled with the feature flag.
- `Candle` backend (`--features candle`): HuggingFace candle runtime;
  `candle_backend.rs` is only compiled with the feature flag.
- Without the feature flag, non-mock backends return an informative
  `OrbokError::Cache` with the feature flag name.
- **Recommended default model: `multilingual-e5-small`** (384-dim,
  Apache 2.0, 100-language support, ~118 MB). Selected because orbok's
  target use case includes mixed Japanese-English documents (RFC-014).
  `RECOMMENDED_HF_MODEL_ID`, `RECOMMENDED_MODEL_DIMENSION`, and
  `recommended_config(weights_path)` documented in the crate.
- Storage impact: 384-dim = 1.5 KiB/chunk (FP32). At 100k chunks: ~147 MB.

**RFC-022 — PDF Extraction Backend**
- `PdfExtractor` in `orbok-extract` using **lopdf** (pure Rust, MIT,
  no C FFI). Selected over pdfium (requires native library) for v0.7.
- Page-level text extraction: each page becomes one `ExtractedSegment`
  with `LocationQuality::PageOnly` (honest; line numbers unavailable).
  UI must not show false line numbers for PDF results.
- Failure isolation: per-page errors are swallowed; one bad page never
  stops extraction of the rest of the document (RFC-005 §13).
- Encrypted PDF → `EncryptedDocument` error category.
- Scanned/image-only PDF → zero segments, no error.
- `PdfExtractor` registered in `ExtractorRegistry` for `.pdf` extension.
- Japanese UTF-8 PDFs extract correctly; legacy SJIS/EUC not attempted.

**RFC-029 — Model Download Integrity and Trust**
- `verify_model_sha256(path, expected_hash)` in orbok-db: streams the
  model file and compares against a user-provided SHA-256 hex string.
- Returns `Ok(true)` on match, `Ok(false)` on mismatch, `Err` on I/O
  error. Path is not logged (NFR-014).
- `ModelRepository::locate()` registers an existing on-disk model file
  (manual placement, no automatic download — RFC-029 §9).
- `models.license_summary` stores the license string shown to the user
  before a model is used.
- `InferenceBackend` enum and `EmbeddingModelConfig`/`RerankerConfig`
  types added to `orbok-models` for full config-driven backend selection.

### Tests
- `orbok-embed`: 4 tests (mock backend, feature-flag error, defaults).
- `orbok-extract`: 29 tests (+14 covering RFC-021/022/029).
- Workspace total: **142 tests / 0 failures**.

### RFCs
- RFC-021, RFC-022, RFC-029 moved from `rfcs/draft/` to `rfcs/done/`.
- 26 of 31 RFCs now in `done/`.

---

## [0.8.0] — 2026-06-07 — All RFCs resolved

> **v1.0.0 is not yet released.** This release completes every RFC
> in the design set. v1.0.0 requires explicit project owner confirmation
> after the three release gate conditions are verified.

### Benchmark Results (RFC-016)

Measured on 100 synthetic documents (debug profile, keyword-only):

| Metric | Result | v1.0 Gate |
|---|---|---|
| Indexing throughput | 59.2 files/s | — |
| Search p99 | 31.18 ms | ≤ 200 ms ✓ |
| Recall@5 (keyword-only) | 75.0% | ≥ 75% ✓ |

Both v1.0.0 search performance gates pass even in the conservative
debug-profile, keyword-only configuration. With a real embedding model
in release mode, both metrics will improve further.

### Added

**RFC-023 — ANN decision documented**
- Measured exact cosine scan baseline: p99 < 35 ms at 100 documents
  (debug mode). ANN complexity is not justified at current scale.
- Decision: keep exact scan for v1.0.0; implement HNSW only when
  user corpora show > 200 ms p99 (tracked as future work).
- `bench_full_pipeline` test runs 100-document benchmark as a
  regression gate for search performance.

**RFC-024 — INT8 vector quantization**
- `quantize_to_i8`, `dequantize_from_i8`, `i8_vec_to_blob`,
  `i8_blob_to_vec`, `cosine_similarity_i8` in orbok-models.
- Storage impact: 4× smaller than FP32 (384 bytes vs 1,536 bytes/chunk).
  At 100k chunks: ~37 MB (INT8) vs ~147 MB (FP32).
- Quality loss measured: cosine similarity error < 0.02 for
  L2-normalized 384-dim vectors.
- `EmbeddingRepository::upsert_i8` stores INT8 vectors with
  `vector_format = 'int8'`; `list_active_i8_for_scan` dequantizes
  on read for exact cosine search.
- INT8 is the Space Saving mode default; Balanced/High Accuracy
  keep FP32.

**RFC-025 — Scanned document detection**
- `is_scanned_pdf(output, page_count)` in orbok-extract::pdf:
  returns `true` when a PDF has pages but zero extracted text.
- `pdf_page_count(path)` helper for the detection check.
- Clear `char_count = 0` signal enables the UI to show an
  "OCR required" notice. Full OCR engine integration deferred.

**RFC-028 — Plugin extractor architecture**
- `PluginManifest` struct: `plugin_id`, `display_name`, `extensions`,
  `author`, `license`, `builtin`, `privacy_note`.
- `PluginExtractor` wrapping a `DocumentExtractor` with its manifest.
- `PluginRegistry::default()` registers all built-in extractors
  (markdown, plain-text, pdf-lopdf) with proper manifests.
- Security contract documented: plugins receive only `ValidatedPath`;
  dynamic loading deferred until RFC-028 is fully activated.

**RFC-030 — Portable mode**
- `--portable` flag: stores catalog and cache in `./orbok-data/`
  instead of the platform app-data directory.
- `data_dir_for_args(portable)` in bootstrap resolves the correct
  path.
- Standard mode remains the default; portable mode is explicit.

**RFC-026 — Archived**
- Encrypted local indexes require a dedicated key-management security
  audit and are not suitable for pre-v1.0.0 implementation.
- RFC-026 moved to `rfcs/archive/` with rationale.

### Tests
- `orbok-models`: 11 tests (+4 quantization tests).
- `orbok-workers`: 56 tests (+10 covering v0.8 RFCs).
- `orbok-bench`: 1 integration test (full 100-doc pipeline benchmark).
- Workspace total: **157 tests / 0 failures**.

### RFC Status
- `rfcs/done/`: 31 RFCs
- `rfcs/archive/`: 1 RFC (RFC-026)
- `rfcs/draft/`: 0 (empty)
- `rfcs/proposed/`: 0 (empty)

---

## [0.9.0] — 2026-06-07 — Release Candidate

> **v1.0.0 not yet released.** This is the release candidate.
> v1.0.0 requires explicit project owner confirmation.

### Added

**DOCX extractor** (`orbok-extract/src/docx.rs`)
- Microsoft Word 2007+ (`.docx`) files extracted via ZIP+XML parsing.
- Reads `word/document.xml`, recovers paragraph text from `<w:t>` runs.
- `LocationQuality::Approximate` (paragraph order preserved; no byte offsets).
- Registered in `ExtractorRegistry` and `PluginRegistry`.
- Failure-isolated: parse errors return typed `ParserError`, no panic.

**HTML extractor** (`orbok-extract/src/html.rs`)
- HTML/HTM files extracted via pure state-machine tag stripper.
- Block-level elements (`<p>`, `<div>`, `<h1>`–`<h6>`, `<li>`, etc.) produce paragraph breaks.
- `<h1>`–`<h6>` headings tracked in `heading_path` (e.g. "Guide > Install").
- `<script>`, `<style>`, `<head>` content suppressed.
- Common entities decoded (`&amp;`, `&lt;`, `&gt;`, `&nbsp;`, `&quot;`).
- `LocationQuality::Approximate`.
- Registered for `.html` and `.htm`.

**End-to-end pipeline integration test**
- `e2e_full_pipeline_write_scan_index_search` in v09_rc:
  writes Markdown + HTML files, runs scan → extract → index → search,
  then verifies:
  - `ERR-4042` found and ranked first in `auth.md`
  - `snippet cache cleanup` returns results
  - HTML `client_secret` content is indexed and searchable

**Pre-release gate tests**
- `all_documented_file_types_have_extractor`: every extension claimed in
  `docs/src/users/file_types.md` has a registered extractor.
- `plugin_registry_all_extractors_have_privacy_notes`: all 5 plugins
  (markdown, docx, html, plain-text, pdf) have license + privacy note.
- `startup_recovery_clean_on_fresh_catalog`: RFC-018 recovery path.
- `pipeline_leaves_no_running_jobs_after_completion`: clean shutdown
  contract (no jobs stuck in `running`).

### Fixed
- **HTML skip-depth bug**: nested `<style>` inside `<head>` incremented
  `skip_depth` without a matching decrement, causing the entire document
  body to be silently skipped. Fixed: nested skip-depth only counts
  same-tag nesting (e.g. `<head>…<head>…</head>…</head>`).
- **Heading detection order**: closing `</h1>` was matched by the
  generic BLOCK_TAGS branch before reaching the heading branch, emitting
  headings as plain paragraphs. Fixed by checking heading close tags
  first in the dispatch chain.
- All 6 compiler warnings across orbok-search, orbok-extract,
  orbok-workers resolved. Build is warning-free.

### Tests
- `orbok-extract`: 29 tests (DOCX and HTML covered by v09_rc in
  orbok-workers, which is the integration host).
- `orbok-workers`: 68 tests (+12 covering DOCX, HTML, E2E pipeline,
  and pre-release gates).
- Workspace total: **169 tests / 0 failures / 0 warnings**.

---

## [0.9.1] — 2026-06-07 — Startup wizard + settings integration

### Added

**OrbokSettings** (`orbok-app/src/settings.rs`)
- `OrbokSettings` struct: `embedding_model_dir`, `reranker_model_dir`,
  `index_mode`, `locale`, `rerank_enabled`, `background_indexing`,
  `pause_on_battery`.
- `load_settings()` / `save_settings()` via `app-json-settings` v2
  (`ConfigManager<OrbokSettings>::new().with_filename("settings.json")`).
- Note in code: a `.with_app_name("orbok")` builder would guarantee
  consistent config paths when binary name differs — flagged for the
  crate author to consider.

**Model verifier** (`orbok-workers/src/model_verifier.rs`)
- `verify_embedding_model(model_dir: Option<&str>) -> VerifyOutcome`
  checks `onnx/model.onnx` and `tokenizer.json` for existence and
  size > 0. Runs in < 2 ms at startup (no SHA-256 hashing).
- `VerifyOutcome`: `Ready`, `NotConfigured`, `FilesInvalid { model_dir, issues }`.
- `FileIssue` with `FileIssueKind`: `NotFound`, `Empty`, `PermissionDenied`.
- `verify_outcome_summary()`: log-safe string that never includes paths.
- 7 unit tests covering all outcomes.

**Startup wizard UI** (`orbok-ui`)
- `WizardState` enum in `state.rs`: `NotConfigured`, `FileMissing`,
  `Checked`, `Ready`.
- `WizardFileCheck` struct: relative path, found, size_mb.
- New messages: `WizardPathChanged`, `WizardValidate`, `WizardChecked`,
  `WizardAccept`, `WizardSkip`.
- `views/wizard.rs`: four page functions (`page_input`, `page_checked`,
  `page_ready`) covering all wizard states.
- 18 new `MessageKey` variants with English + Japanese translations.
- `shell.rs`: wizard takes priority over normal navigation — when
  `state.wizard.is_some()`, the wizard is shown instead of the shell.

**Bootstrap update** (`orbok-app/src/bootstrap.rs`)
- `load_initial_state()` now:
  1. runs RFC-018 startup recovery
  2. loads `OrbokSettings`
  3. calls `verify_embedding_model`
  4. sets `wizard = Some(WizardState::NotConfigured)` on first launch
  5. sets `wizard = Some(WizardState::FileMissing { previous_dir })` when
     files are gone
  6. sets `capability = Hybrid` only when `VerifyOutcome::Ready`
- `persist_model_dir(dir)`: writes accepted model directory back to
  `OrbokSettings` via `save_settings`.
- `--check` output now includes model verification status.

**main.rs backend effects**
- `WizardValidate`: runs `verify_embedding_model` on the input path,
  builds file check results, dispatches `WizardChecked`.
- `WizardAccept`: calls `persist_model_dir` to write the accepted path
  to `settings.json` before the UI transitions to full mode.

### Tests
- `orbok-workers`: 75 tests (+7 model_verifier).
- Workspace total: **175 tests / 0 failures**.

---

## [0.9.2] — 2026-06-07 — Source management + hybrid search wiring

### Added

**EmbeddingWorker model selection**
- `EmbeddingWorker::with_model(catalog, cache, model, model_id)` —
  constructor accepting any `Box<dyn EmbeddingModel>`. Tests can pass
  `MockEmbeddingModel`; production builds pass the factory result from
  `orbok_embed::create_embedding_model`.

**HybridSearchService in bootstrap** (`run_search`)
- `bootstrap::run_search` now uses `HybridSearchService` throughout.
- When `OrbokSettings.embedding_model_dir` is set: calls
  `orbok_embed::create_embedding_model` with a `recommended_config`.
  If the `tract` feature is compiled and the model file exists, real
  semantic search is used. Otherwise falls back to keyword-only with
  no error — the capability degradation is logged at `warn` level.

**Source management backend**
- `bootstrap::add_source(catalog, path)` — resolves tilde, canonicalizes,
  inserts source record, returns `SourceCard`.
- `bootstrap::scan_and_index_source(catalog, cache, source_id)` — runs
  `Scanner` → `ExtractionWorker` → `ChunkAndIndexWorker` synchronously,
  returns updated `IndexHealth`.
- `bootstrap::remove_source(catalog, source_id)` — calls
  `delete_with_all_data`.
- `bootstrap::get_health(catalog)` — queries `count_with_status` across
  all file statuses; populates `IndexHealth`.
- `bootstrap::get_sources(catalog)` — loads all sources with per-source
  indexed/stale/failed counts.

**FileRepository count methods** (`orbok-db`)
- `count_with_status(status)` — global file count by status.
- `count_for_source_with_status(source_id, status)` — source-scoped count.

**Sources view** (`orbok-ui`)
- Path text-input always visible: user types/pastes a folder path and
  presses Enter or clicks the button to add a source.
- Per-source Remove button dispatches `Message::SourceRemoved(source_id)`.
- `Message::SourcePathChanged`, `RequestAddSource`, `SourceAdded`,
  `SourceRemoved`, `ScanCompleted`, `HealthUpdated`, `SourcesLoaded`
  added to the message vocabulary.
- `SourceCard.source_id: String` — backend ID field for remove operations.

**Startup population**
- `load_initial_state` now populates `AppState.health` and
  `AppState.sources` from the catalog at startup, so the Indexing
  sidebar and Sources view show real data immediately.

### Tests
- `orbok-workers`: 84 tests (+9 covering source management, health
  queries, EmbeddingWorker model selection, hybrid search routing).
- Workspace total: **184 tests / 0 failures**.

---

## [0.9.3] — 2026-06-07 — Dependency hardening

### Changed

**`lopdf` upgraded: 0.34.0 → 0.41.0** (`orbok-extract`)
Seven minor versions. All existing `Document::load` / `page_iter` /
`extract_text` / `get_pages` APIs are unchanged (upstream explicitly
guarantees backward compatibility). New capabilities available to orbok:
PDF 1.5+ object streams (enables reading compressed modern PDFs that
previously surfaced zero-length text), improved XRef stream handling,
and Rust 2024 edition alignment. Requires Rust ≥ 1.85, which orbok already
targets.

**`sha2` upgraded: 0.10.9 → 0.11.0** (workspace)
The sha2 0.11.x series adopts the `digest 0.11` crate, which switches
internal output types from `GenericArray<u8, N>` (generic-array 0.14) to
`Array<u8, N>` (hybrid-array). Two call sites that formatted digests with
`format!("{:x}", …)` were migrated to an explicit byte-iterator collect —
semantically identical, one fewer implicit trait dependency. sha2 0.10.9
is still present as a transitive dep (locked by the cryptography dep
chain); both versions coexist cleanly.

**`orbok-workers` test isolation**
The `orbok-ui` dev-dependency was removed from `orbok-workers`. Tests that
previously imported `orbok_ui::state::{AppState, Message}` to verify UI
invariants were either stubbed with equivalent non-GUI assertions (the
logical property is preserved) or noted as covered by `orbok-ui`'s own
suite. This eliminates the iced → winit → wayland/x11 compile chain from
the non-GUI test run, cutting `cargo test` peak disk use by ~9 GB.

**Dependency audit** (full results in `docs/src/maintainers/dep_audit.md`)
- All other workspace deps verified current as of 2026-06-07
- `zip = "2"` spec intentional; zip 8.x is a breaking API rewrite
- `candle-core`: 0.9.2 → 0.10.2 available; deferred to `--features candle`
  activation milestone
- `localcache`, `app-json-settings`: ask the author (nabbisen) directly

### Tests
**184 tests / 0 failures** (unchanged count; test logic improved).

---

## [0.9.4] — 2026-06-08 — Candle upgrade + lucide-icons integration

### Changed

**`candle-core` / `candle-nn` upgraded: 0.9.2 → 0.10.2** (`orbok-embed`,
`--features candle`)
Drop-in upgrade per migration report: no API symbols removed, one addition
each (`TokenizerFromGguf` in candle-core, `remove_mean` in candle-nn),
neither relevant to orbok's CPU inference path. Source unchanged.

**lucide-icons added: 1.17.0** (`orbok-ui`)
snora 0.8.0 ships a native `lucide-icons` feature (`Icon::Lucide` variant).
Enabling it via `snora = { features = ["lucide-icons"] }` activates full
Lucide icon support in the sidebar navigation rail and anywhere else an
iced widget tree is built.

Icon font registration — `orbok-ui` re-exports `LUCIDE_FONT_BYTES`; the
iced application builder in `orbok-app` registers it via `.font()` at
startup so all icon glyphs render correctly.

**Sidebar navigation** now uses proper Lucide icons instead of emoji:

| View | Icon |
|---|---|
| Search | `Search` |
| Sources | `FolderOpen` |
| Indexing | `ListOrdered` |
| Storage | `Database` |
| Models | `Cpu` |
| Settings | `Settings` |

**In-page icon buttons** (views.rs, wizard.rs):
- Search submit button — `icon_search` + label
- Add Source button — `icon_folder_plus` + label
- Remove source — `icon_trash_2` (icon-only, compact)
- Wizard Validate — `icon_scan_eye` + label
- Wizard Accept — `icon_circle_check` + label

### Tests
**184 tests / 0 failures.** No new tests (icon rendering is a visual
concern; the logic under the buttons is unchanged and already covered).

---

## [0.9.5] — 2026-06-08 — Navigation restructure + UX fixes

### Changed

**Navigation: two-level layout (sidebar groups + tab bar)**

The six flat sidebar items are replaced with three top-level groups and
per-group sub-tabs, following the approved hierarchy:

| Group | Sidebar icon | Tabs |
|---|---|---|
| Search | `LucideIcon::Search` | Search · Sources |
| AI | `LucideIcon::BrainCircuit` | Indexing · Storage · Models |
| Settings | `LucideIcon::Settings` | (single page) |

`NavGroup` enum added to `orbok-ui::state`. `ViewId::group()` maps any
view to its parent group. `ViewId::group_default()` gives the default
tab when entering a group. snora's `TabBar` / `app_tab_bar` render the
horizontal tab strip. The `SwitchGroup(NavGroup)` message activates the
default tab for a group.

**Add Folder — native OS folder picker (`rfd 0.15`)**
Clicking "Add Folder" now opens the operating system's native folder
picker dialog. No path typing required. The selected path is scanned and
indexed immediately. The manual path text-input field remains as a
fallback for power users who prefer to type or paste a path.

**Sources view — recursive scanning note**
A subtitle line "All sub-folders are scanned recursively." appears below
the add-folder controls, answering the immediate question new users have
about search scope.

### Tests
**184 tests / 0 failures.**

---

## [0.9.6] — 2026-06-08 — Crate directory restructure

### Changed

The twelve crates that were flat in `crates/` are now grouped into
logical subdirectories. Package names and all Rust `use` paths are
unchanged — only filesystem paths and the workspace `Cargo.toml` member
entries differ.

```
crates/
├── app/                 # orbok-app   — binary, bootstrap, settings
├── bench/               # orbok-bench — benchmark harness
├── core/                # orbok-core  — IDs, errors, lifecycle types
├── data/
│   ├── cache/           # orbok-cache — localcache wrapper
│   ├── catalog/         # orbok-db    — SQLite schema, repos, migrations
│   └── fs/              # orbok-fs    — scanner, path guard, hashing
├── pipeline/
│   ├── extract/         # orbok-extract — extractors, chunker
│   └── workers/         # orbok-workers — indexing pipeline, recovery
├── search/
│   ├── embed/           # orbok-embed  — inference backends
│   ├── engine/          # orbok-search — FTS5, vector, hybrid RRF
│   └── models/          # orbok-models — model traits, mocks
└── ui/                  # orbok-ui   — snora/iced shell, views, i18n
```

184 tests / 0 failures.
