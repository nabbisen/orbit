# orbok Implementation Roadmap

See `orbok-roadmap-v1.md` (project files) for the full rationale.
This file tracks the milestone status as of each release.

## Milestone Status

| M | Name | v0.1 |
|---|---|:---:|
| M0 | Project Skeleton and Architecture Boundaries | ✓ |
| M1 | Local Data Lifecycle and SQLite Catalog | ✓ |
| M2 | Source Registration and Safe File Access | ✓ |
| M3 | File Scanner and Change Detection | ✓ |
| M4 | Document Extraction Pipeline | ✓ |
| M5 | Adaptive Chunking and Location Metadata | — |
| M6 | Keyword Search MVP | Prototype |
| M7 | Embedding and Vector Search MVP | — |
| M8 | Hybrid Search and RRF | — |
| M9 | Search UI MVP | Shell only |
| M10 | Storage Dashboard and Cleanup | Partial |
| M11 | Optional Reranking | — |
| M12 | Model Registry and Installation UX | Types only |
| M13 | Hardening, Benchmarks, and Packaging | — |

## Next Steps (v0.2 targets)

- **M5** Chunking: paragraph/heading-aware chunker, byte/line offsets,
  parent-child chunk model.
- **M6 complete**: connect extraction output to the keyword engine,
  surface results in the Search view.
- **M7** Embeddings: candle or ONNX Runtime inference, per-file bundles
  via orbok-cache, vector store.
- **M8** Hybrid + RRF: fuse keyword and vector candidates.
- **M9** Search results: result cards, snippet loading from source files.
- **RFC-014** Japanese tokenization strategy.

GUI note: the snora framework selection is finalized in RFC-027
(iced 0.14, no WebView, no local HTTP API). The GUI crate boundary
is enforced: orbok-ui never reads the filesystem directly.

Cache engine note: localcache 0.20.0 is pinned (mtime nanosecond
precision fix). rusqlite 0.40 is pinned to maintain a single
libsqlite3-sys instance across localcache and orbok-db.
