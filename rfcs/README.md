# orbok RFCs

Lifecycle policy: [RFC 000](./done/000-rfc-lifecycle-policy.md).
The folder an RFC lives in is the source of truth for its state.

## Proposed

Foundation (Part 1):

| ID | Title | Milestone |
|----|-------|-----------|
| 001 | [Local Data Classification and Lifecycle](./proposed/001-local-data-classification-and-lifecycle.md) | M1 |
| 002 | [SQLite Catalog Schema and Migration Policy](./proposed/002-sqlite-catalog-schema-and-migration-policy.md) | M1 |
| 003 | [Source Registration and File Access Boundary](./proposed/003-source-registration-and-file-access-boundary.md) | M2 |
| 004 | [File Scanner and Change Detection](./proposed/004-file-scanner-and-change-detection.md) | M3 |
| 005 | [Document Extraction Pipeline](./proposed/005-document-extraction-pipeline.md) | M4 |

Retrieval core (Part 2):

| ID | Title | Milestone |
|----|-------|-----------|
| 006 | [Adaptive Chunking and Location Metadata](./proposed/006-adaptive-chunking-and-location-metadata.md) | M5 |
| 007 | [Keyword Search Engine Selection](./proposed/007-keyword-search-engine-selection.md) | M6 |
| 008 | [Embedding Model and Vector Storage](./proposed/008-embedding-model-and-vector-storage.md) | M7 |
| 009 | [Hybrid Search and RRF Fusion](./proposed/009-hybrid-search-and-rrf-fusion.md) | M8 |
| 010 | [Optional Local Reranking](./proposed/010-optional-local-reranking.md) | M11 |

Product UX, operations, security (Part 3):

| ID | Title | Milestone |
|----|-------|-----------|
| 011 | [Storage Dashboard and Cleanup UX](./proposed/011-storage-dashboard-and-cleanup-ux.md) | M10 |
| 012 | [Model Registry and Installation Workflow](./proposed/012-model-registry-and-installation-workflow.md) | M12 |
| 013 | [Search View and Result Explanation UX](./proposed/013-search-view-and-result-explanation-ux.md) | M9 |
| 014 | [Japanese and Mixed-Language Search Strategy](./proposed/014-japanese-and-mixed-language-search-strategy.md) | M6/M8 |
| 015 | [Security Hardening for Local Files and Local API](./proposed/015-security-hardening-for-local-files-and-local-api.md) | M13 |

Release readiness (Part 4):

| ID | Title | Milestone |
|----|-------|-----------|
| 016 | [Benchmark and Retrieval Evaluation Plan](./proposed/016-benchmark-and-retrieval-evaluation-plan.md) | M13 |
| 017 | [Packaging and Distribution Strategy](./proposed/017-packaging-and-distribution-strategy.md) | M13 |
| 018 | [Crash Recovery, Diagnostics, and Repair Tools](./proposed/018-crash-recovery-diagnostics-and-repair-tools.md) | M13 |
| 019 | [Test Matrix and Release Readiness](./proposed/019-test-matrix-and-release-readiness.md) | M13 |
| 020 | [Documentation and User Guidance Structure](./proposed/020-documentation-and-user-guidance-structure.md) | M13 |

Activated / added after package review (2026-06-06):

| ID | Title | Milestone |
|----|-------|-----------|
| 027 | [GUI Framework Finalization — iced via snora](./proposed/027-gui-framework-finalization.md) | M0 / M9 |
| 031 | [GUI Internationalization (i18n)](./proposed/031-gui-internationalization.md) | M0 / M9 |

## Implemented

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | repo bootstrap |

## Draft (deferred future RFCs)

Not part of the implementation baseline; see activation conditions in
each file. Do not implement speculatively.

| ID | Title | Reconsider when |
|----|-------|-----------------|
| 021 | [Default Embedding Model Selection](./draft/021-default-embedding-model-selection.md) | Embedding pipeline + benchmark corpus exist |
| 022 | [PDF Extraction Backend Selection](./draft/022-pdf-extraction-backend-selection.md) | Baseline extraction + PDF fixtures exist |
| 023 | [Vector ANN Indexing](./draft/023-vector-ann-indexing.md) | Exact search latency proven insufficient |
| 024 | [Vector Quantization](./draft/024-vector-quantization.md) | Vector storage size measured on real corpora |
| 025 | [OCR Pipeline](./draft/025-ocr-pipeline.md) | Text search stable, scanned-doc demand confirmed |
| 026 | [Encrypted Local Indexes](./draft/026-encrypted-local-indexes.md) | Key-management requirements clarified |
| 028 | [Plugin Extractor Architecture](./draft/028-plugin-extractor-architecture.md) | Built-in extractors stabilized |
| 029 | [Model Download Integrity and Trust Policy](./draft/029-model-download-integrity-and-trust-policy.md) | Before enabling model downloads |
| 030 | [Portable Mode](./draft/030-portable-mode.md) | Portable distribution becomes a goal |

## Archive

(empty)

## Appendices

| Appendix | Title |
|----------|-------|
| A | [localcache Integration Policy](./appendices/APPENDIX-A-localcache-integration.md) |

## Change log (RFC directory)

- 2026-06-06 — Adopted RFC-000 structure; migrated RFCs 001–030 from the
  five package tarballs into `proposed/` (001–020) and `draft/`
  (021–026, 028–030). Renamed `RFC-NNN-slug.md` → `NNN-slug.md`
  (numbers unchanged).
- 2026-06-06 — Activated RFC-027 with the project-owner decision:
  GUI framework is **iced 0.14 via snora v0.8** (no WebView, no local
  HTTP API). Moved `draft/` → `proposed/`.
- 2026-06-06 — Added RFC-031 (GUI Internationalization) covering the
  project-instruction i18n requirement.
- 2026-06-06 — Amendments: Appendix A pinned to localcache v0.19.1 and
  rusqlite-0.40 alignment; RFC-002 rusqlite pin; RFC-015 local-API
  sections marked dormant under the native-GUI decision.
