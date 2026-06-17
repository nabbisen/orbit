# Architecture Overview

orbok is a Rust workspace of nine crates.

## Crate Map

- **orbok-core** — typed IDs, error types, data-lifecycle classes, status vocabulary
- **orbok-db** — SQLite catalog: migrations, repositories (RFC-002)
- **orbok-fs** — safe file access boundary, source policies, scanner (RFC-003/004)
- **orbok-cache** — localcache wrapper for derived-data payloads (Appendix A)
- **orbok-extract** — extractor trait, text/markdown extractors, normalization (RFC-005)
- **orbok-search** — keyword engine trait and FTS5 implementation (RFC-007)
- **orbok-models** — model vocabulary and capability summary (RFC-012)
- **orbok-ui** — snora/iced views, i18n catalog, navigation shell (RFC-027/031)
- **orbok-app** — binary: bootstrap, `--check` mode, GUI launch

## Key Design Rules

1. **orbok-ui never accesses the filesystem** (RFC-027).
2. **Every file read goes through PathGuard** (RFC-003 §8).
3. **The catalog is authoritative**; localcache payloads live in a separate DB (Appendix A §3).
4. **Cleanup runs only from a validated CleanupPlan** (RFC-001 §14).
