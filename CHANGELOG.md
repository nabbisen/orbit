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
