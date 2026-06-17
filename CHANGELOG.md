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
