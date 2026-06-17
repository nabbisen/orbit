# orbok Implementation Roadmap

## Milestone Status

| M | Name | v0.1 | v0.2 |
|---|---|:---:|:---:|
| M0 | Project Skeleton and Architecture Boundaries | ✓ | |
| M1 | Local Data Lifecycle and SQLite Catalog | ✓ | |
| M2 | Source Registration and Safe File Access | ✓ | |
| M3 | File Scanner and Change Detection | ✓ | |
| M4 | Document Extraction Pipeline | ✓ | |
| M5 | Adaptive Chunking and Location Metadata | | ✓ |
| M6 | Keyword Search MVP | Proto | ✓ |
| M7 | Embedding and Vector Search MVP | | |
| M8 | Hybrid Search and RRF | | |
| M9 | Search UI MVP | Shell | Partial |
| M10 | Storage Dashboard and Cleanup | Partial | |
| M11 | Optional Reranking | | |
| M12 | Model Registry and Installation UX | Types | |
| M13 | Hardening, Benchmarks, and Packaging | | |

## Next (v0.3 targets)

### M7 — Embeddings and Vector Search

- `EmbeddingModel` trait + mock implementation (deterministic, test-safe).
- `EmbeddingWorker` in `orbok-workers`: reads chunk text from extraction
  cache, generates embeddings, stores them in the `embeddings` table.
- Exact cosine-similarity scan (no ANN in v0.3; dataset sizes are small).
- Vector storage as `sqlite_blob` in the catalog embeddings table.
- Model version tracking: changing the embedding model marks existing
  embeddings stale.
- **RFC-008** implementation target.

### M8 — Hybrid Search and RRF

- `HybridSearchService` merging keyword and vector candidates.
- Reciprocal Rank Fusion (k=60, configurable) — RFC-009.
- Candidate deduplication by chunk or parent context.
- Result explanation badges: Keyword / Semantic / Fused.
- Search mode selector in `orbok-ui` (Auto / Exact / Conceptual).

### M9 (complete) — Search UI

- Result preview panel with "why this result" explanation.
- Stale/missing source badges on result cards.
- Filter drawer (source, file type, date range).
- Open file / open folder actions wired to `orbok-app`.
- **RFC-013** implementation target.

### Other v0.3 candidates

- Persist locale preference to catalog settings on change.
- Source health banner in the UI (stale/missing file counts).
- Scan-on-startup option (configurable via settings).
- Storage accounting populated after index runs.
- RFC-014 scoping: evaluate unicode61 trigram vs Tantivy for Japanese.

## Design decisions (settled)

- **GUI**: iced 0.14 via snora 0.8 — no WebView, no local HTTP API (RFC-027).
- **i18n**: compile-time typed catalog, En+Ja (RFC-031).
- **DB pin**: localcache 0.20.0 + rusqlite 0.40 — one libsqlite3-sys (RFC-002 §16).
- **FTS**: SQLite FTS5 contentless + `keyword_index_records.fts_rowid` mapping (RFC-007).
- **Chunking**: structure-aware (Markdown headings) + paragraph fallback (RFC-006).
- **Pipeline**: extract → chunk+index in two worker steps, atomic per-file transactions (RFC-006 §12).

## v0.4 status

| RFC | Title | v0.4 |
|---|---|:---:|
| RFC-010 | Optional Local Reranking | ✓ |
| RFC-011 | Storage Dashboard and Cleanup UX | ✓ |
| RFC-013 | Search View and Result Explanation UX | ✓ |
| RFC-014 | Japanese and Mixed-Language Search | ✓ |

## v0.5 targets

- **RFC-012**: Model Registry and Installation Workflow — full model registry UI, locate/install/validate model files, reindex-on-change flow.
- **RFC-015**: Security Hardening — CSRF protection for local API (when applicable), path-traversal audit, HTML render sanitization hardening.
- **RFC-016**: Benchmarks and Retrieval Evaluation — search quality test corpus, indexing throughput, memory profiling.
- **RFC-017**: Packaging and Distribution — cross-platform release binaries, Debian/RPM packages, macOS .app bundle, Windows installer.
- **M9 complete**: Two-pane preview panel with full explanation (RFC-013 follow-through), file-open OS integration in orbok-app.
- **M10 complete**: Storage dashboard cleanup actions wired end-to-end (CleanupService combining catalog + cache).
