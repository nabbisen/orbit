# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.1.0] — 2026-06-07

### Added

**Foundation (M0–M1)**
- Rust 2024 edition Cargo workspace with nine crates:
  `orbok-core`, `orbok-db`, `orbok-fs`, `orbok-cache`,
  `orbok-extract`, `orbok-search`, `orbok-models`, `orbok-ui`, `orbok-app`.
- RFC lifecycle policy (RFC-000): `rfcs/{proposed,done,archive}` folder
  structure with numbered RFCs, README index.
- RFC-001: three-layer data lifecycle (persistent catalog / rebuildable
  index / ephemeral cache).
- RFC-002: SQLite catalog schema with append-only migrations, FTS5
  contentless keyword index, foreign-key enforcement.

**Source boundary (M2)**
- RFC-003: source registration, canonical path enforcement, symlink
  policy (Ignore / FollowWithinSource / FollowAllWithWarning), hidden-file
  policy, sensitive-directory warnings (.ssh, .gnupg, .aws …).

**File scanner (M3)**
- RFC-004: recursive directory walker, RFC 3339 mtime comparison
  (nanosecond precision — no same-second-overwrite blind spot),
  SHA-256 content hashing, stale/missing/discovered state machine,
  per-file failure isolation, cancellation support, index-job queueing.

**Extraction (M4)**
- RFC-005: extractor trait, plain-text and Markdown extractors with
  line-aware offsets, normalization pipeline (unicode NFC, whitespace,
  newlines), extractor version tracking.

**Cache engine (Appendix A)**
- localcache 0.20.0 integration: namespace policy, MetadataThenFullHash
  change detection, TTL and LRU controls, plan-validated cleanup.
- Namespace map: `extract-segments:v1`, `chunk-bundle:v1`,
  `embedding-bundle:<model>:<fmt>:v1`, `preview-cache:v1`.

**Keyword search (M6 prototype)**
- RFC-007: FTS5 contentless engine behind `KeywordSearchEngine` trait;
  safe query building (FTS5 operator neutralization, RFC-015);
  replace-on-reindex, deletion via `contentless_delete=1`.

**GUI and i18n (RFC-027, RFC-031)**
- snora 0.8 (iced 0.14) application shell with six-page sidebar
  navigation.
- Typed i18n catalog: English and Japanese, exhaustive at compile time.
- Headless `--check` mode for CI and display-less environments.

### Dependencies (pinned)
- localcache 0.20.0 (mtime nanosecond precision, schema v5).
- rusqlite 0.40 (matching localcache; single libsqlite3-sys instance).
- iced 0.14 via snora 0.8.
