# orbok RFC Index

Managed by RFC-000. Last updated: 2026-06-07.

## Implemented

| ID | Title | Release |
|---|---|---|
| 000 | [— RFC lifecycle policy](done/000-rfc-lifecycle-policy.md) | v1.4.0 |
| 001 | [Local Data Classification and Lifecycle](done/001-local-data-classification-and-lifecycle.md) | v0.1.0 |
| 002 | [SQLite Catalog Schema and Migration Policy](done/002-sqlite-catalog-schema-and-migration-policy.md) | v0.1.0 |
| 003 | [Source Registration and File Access Boundary](done/003-source-registration-and-file-access-boundary.md) | v0.1.0 |
| 004 | [File Scanner and Change Detection](done/004-file-scanner-and-change-detection.md) | v0.1.0 |
| 005 | [Document Extraction Pipeline](done/005-document-extraction-pipeline.md) | v0.1.0 |
| 006 | [Adaptive Chunking and Location Metadata](done/006-adaptive-chunking-and-location-metadata.md) | v0.2.0 |
| 007 | [Keyword Search Engine Selection](done/007-keyword-search-engine-selection.md) | v0.2.0 |
| 008 | [Embedding Model and Vector Storage](done/008-embedding-model-and-vector-storage.md) | v0.3.0 |
| 009 | [Hybrid Search and RRF Fusion](done/009-hybrid-search-and-rrf-fusion.md) | v0.3.0 |
| 010 | [Optional Local Reranking](done/010-optional-local-reranking.md) | v0.4.0 |
| 011 | [Storage Dashboard and Cleanup UX](done/011-storage-dashboard-and-cleanup-ux.md) | v0.4.0 |
| 012 | [Model Registry and Installation Workflow](done/012-model-registry-and-installation-workflow.md) | v0.5.0 |
| 013 | [Search View and Result Explanation UX](done/013-search-view-and-result-explanation-ux.md) | v0.4.0 |
| 014 | [Japanese and Mixed-Language Search Strategy](done/014-japanese-and-mixed-language-search-strategy.md) | v0.4.0 |
| 015 | [Security Hardening for Local Files and Local API](done/015-security-hardening-for-local-files-and-local-api.md) | v0.5.0 |
| 016 | [Benchmark and Retrieval Evaluation Plan](done/016-benchmark-and-retrieval-evaluation-plan.md) | v0.5.0 |
| 017 | [Packaging and Distribution Strategy](done/017-packaging-and-distribution-strategy.md) | v0.5.0 |
| 018 | [Crash Recovery, Diagnostics, and Repair Tools](done/018-crash-recovery-diagnostics-and-repair-tools.md) | v0.5.0 |
| 019 | [Test Matrix and Release Readiness](done/019-test-matrix-and-release-readiness.md) | v0.6.0 |
| 020 | [Documentation and User Guidance Structure](done/020-documentation-and-user-guidance-structure.md) | v0.6.0 |
| 027 | [GUI Framework Finalization](done/027-gui-framework-finalization.md) | v0.1.0 |
| 031 | [GUI Internationalization (i18n)](done/031-gui-internationalization.md) | v0.1.0 |

## Proposed / Deferred

| ID | Title | Notes |
|---|---|---|
| 032 | [Design Token Foundation and Theming](proposed/032-design-token-foundation-and-theming.md) | Snora Design tokens as single styling source; themes (incl. dark, high-contrast). Foundation for 033–035. |
| 033 | [Component Primitive Migration](proposed/033-component-primitive-migration.md) | snora as sole gateway for UI primitives (button/card/chip/progress). Depends on 032. |
| 034 | [Accessibility Conformance](proposed/034-accessibility-conformance.md) | WCAG 2.1 AA: contrast guard, keyboard map, focus, labels, target size. Depends on 032–033. |
| 035 | [Inclusive Design](proposed/035-inclusive-design.md) | Text scale, reduced motion, CVD-safe status, locale-aware formatting, RTL readiness. Depends on 032. |

Developer handoffs for 032–035 live in [`handoffs/`](handoffs/).

## Archive
*(empty)*
