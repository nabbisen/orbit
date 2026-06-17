# Architecture Overview

orbit is a Rust workspace of nine crates.

## Crate Map

- **orbit-core** — typed IDs, error types, data-lifecycle classes, status vocabulary
- **orbit-db** — SQLite catalog: migrations, repositories (RFC-002)
- **orbit-fs** — safe file access boundary, source policies, scanner (RFC-003/004)
- **orbit-cache** — localcache wrapper for derived-data payloads (Appendix A)
- **orbit-extract** — extractor trait, text/markdown extractors, normalization (RFC-005)
- **orbit-search** — keyword engine trait and FTS5 implementation (RFC-007)
- **orbit-models** — model vocabulary and capability summary (RFC-012)
- **orbit-ui** — snora/iced views, i18n catalog, navigation shell (RFC-027/031)
- **orbit-app** — binary: bootstrap, `--check` mode, GUI launch

## Key Design Rules

1. **orbit-ui never accesses the filesystem** (RFC-027).
2. **Every file read goes through PathGuard** (RFC-003 §8).
3. **The catalog is authoritative**; localcache payloads live in a separate DB (Appendix A §3).
4. **Cleanup runs only from a validated CleanupPlan** (RFC-001 §14).
