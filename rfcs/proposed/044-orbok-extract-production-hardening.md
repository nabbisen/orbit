# RFC-044: orbok-extract Production Hardening and Boundary Cleanup

**Project:** orbok  
**Former project name:** orbit  
**RFC:** 044  
**Title:** `orbok-extract` Production Hardening and Boundary Cleanup  
**Status:** Proposed
**Target milestone:** Extraction reliability / core slimming / release hardening  
**Date:** 2026-06-18  
**Primary target crate:** `orbok-extract v0.9`  
**Related RFCs:** RFC-005 Document Extraction Pipeline, RFC-006 Adaptive Chunking and Location Metadata, RFC-007 Keyword Search Engine Selection, RFC-041 Search, Narrow Results, and Browse Around  

---

## 1. Summary

This RFC defines a **narrow production-hardening plan** for `orbok-extract`.

It does **not** redefine the extraction architecture. The current `orbok-extract v0.9` already has the correct high-level boundary:

```text
validated file path
  ↓
DocumentExtractor
  ↓
ExtractOutput
  ↓
segments
  ↓
normalization / chunking / indexing pipeline
```

The crate already provides:

- `DocumentExtractor`;
- `ExtractOutput`;
- `ExtractedSegment`;
- segment kinds;
- location quality;
- extractor registry;
- normalization;
- text, Markdown, PDF, DOCX, and HTML extractors;
- chunking support;
- plugin-manifest scaffolding.

Therefore, this RFC focuses only on what is still needed before `orbok-extract` can be treated as a production-grade extraction foundation:

1. resource limits;
2. structured extraction warnings;
3. parser panic isolation;
4. clearer location semantics;
5. consistent error mapping;
6. crate-boundary cleanup;
7. focused test hardening.

This RFC explicitly rejects replacing `orbok-extract` with the older `findtext-*` crates.

---

## 2. Decision

The accepted direction is:

```text
Keep orbok-extract as the canonical extraction crate.
Do not migrate to findtext-*.
Do not rewrite the extraction boundary.
Harden the current crate.
Clean up dependency direction.
Add production test gates.
```

If work is split into issues, this RFC should serve as the design and acceptance reference.

---

## 3. Motivation

`orbok-extract v0.9` is already architecturally useful, but production use requires stronger guarantees.

Local document extraction is risky because documents may be:

- huge;
- malformed;
- encrypted;
- partially corrupt;
- hostile or unusual;
- encoded unexpectedly;
- old office files;
- scanned PDFs;
- ZIP containers with large or suspicious entries.

A search app must not crash or freeze because one file is bad.

The user-facing experience must remain calm:

```text
Some files could not be prepared.
Other files are still searchable.
```

The implementation must therefore treat extraction failure as a normal recoverable condition.

---

## 4. Goals

- Preserve the existing `DocumentExtractor` architecture.
- Add resource limits to every extractor path.
- Prevent malformed files from crashing extraction workers.
- Add structured warnings for partial or degraded extraction.
- Make source-location meaning explicit enough for downstream UI and snippets.
- Keep extraction independent from database persistence where practical.
- Ensure consistent error mapping across formats.
- Improve tests for malformed, large, and partially readable files.
- Keep built-in extraction small and predictable.
- Avoid adopting `findtext-*` as production dependencies.

---

## 5. Non-Goals

This RFC does not:

- redesign `DocumentExtractor` from scratch;
- replace `lopdf`;
- introduce OCR;
- add XLSX/spreadsheet extraction;
- introduce dynamic plugin loading;
- move to cloud extraction;
- implement keyword search inside `orbok-extract`;
- adopt `findtext-*` crates directly;
- define UI redesign;
- define model download behavior.

This RFC also does not require perfect extraction quality for every format. It requires robust, bounded, honest extraction.

---

## 6. Current Architecture Assessment

Current `orbok-extract v0.9` already provides the core contract:

```rust
pub trait DocumentExtractor: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn supported_extensions(&self) -> &'static [&'static str];
    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput>;
}
```

and output similar to:

```rust
pub struct ExtractOutput {
    pub extractor_name: String,
    pub extractor_version: String,
    pub normalization_version: String,
    pub segments: Vec<ExtractedSegment>,
    pub char_count: u64,
}
```

This is a good extraction-first design.

The remaining problem is not architecture existence. The remaining problem is production trust.

---

## 7. findtext Decision

The earlier `findtext-*` crates are not migration targets.

Classification:

```text
findtext-* = historical prototype / reference code
orbok-extract = canonical production direction
```

Reasons:

- `findtext-*` APIs are keyword-search-first;
- orbok needs extraction-first;
- `orbok-extract` already has the right crate boundary;
- adding `findtext-*` would increase indirection without improving architecture.

Allowed use:

- inspect as reference;
- reuse small implementation ideas after review;
- derive tests or fixtures.

Not allowed as part of this RFC:

- direct production dependency;
- replacement for `orbok-extract`;
- source of public API shape.

---

## 8. Scope of Hardening

This RFC covers the following concrete changes:

```text
P0:
  ExtractLimits
  ExtractWarning
  panic isolation wrapper
  consistent error mapping
  PDF/DOCX/HTML resource limits
  boundary cleanup around orbok-db dependency
  focused tests

P1:
  LocationKind or SourceLocation
  stronger format fixtures
  better DOCX/HTML warnings
  optional encoding policy refinement

P2:
  spreadsheet extraction
  richer DOCX parts
  OCR
  backend replacement
```

---

## 9. Resource Limits

## 9.1. Problem

Some extractors currently read entire files or document parts into memory. This is acceptable for prototypes but risky for production.

Examples of risky input:

- huge plain text logs;
- very large Markdown files;
- PDFs with many pages;
- DOCX files with huge XML entries;
- malformed ZIP containers;
- HTML files with excessive text;
- files crafted to trigger parser worst cases.

## 9.2. Required Type

Add:

```rust
#[derive(Debug, Clone)]
pub struct ExtractLimits {
    pub max_file_bytes: u64,
    pub max_extracted_chars: u64,
    pub max_segments: usize,
    pub max_pdf_pages: usize,
    pub max_docx_xml_bytes: u64,
    pub max_zip_entry_bytes: u64,
    pub max_html_bytes: u64,
}
```

Recommended default values should be conservative and configurable by orbok app settings.

Initial suggestion:

```rust
impl Default for ExtractLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_extracted_chars: 5_000_000,
            max_segments: 20_000,
            max_pdf_pages: 1_000,
            max_docx_xml_bytes: 32 * 1024 * 1024,
            max_zip_entry_bytes: 64 * 1024 * 1024,
            max_html_bytes: 32 * 1024 * 1024,
        }
    }
}
```

These values are not final release policy; they are safe starting points.

## 9.3. Extract Input Change

If the current trait should not change widely, introduce a registry-level context first:

```rust
pub struct ExtractContext {
    pub limits: ExtractLimits,
}
```

Preferred long-term trait:

```rust
fn extract(&self, path: &ValidatedPath, context: &ExtractContext) -> OrbokResult<ExtractOutput>;
```

If trait migration is too disruptive, provide:

```rust
fn extract_with_limits(
    &self,
    path: &ValidatedPath,
    limits: &ExtractLimits,
) -> OrbokResult<ExtractOutput>;
```

and keep `extract` as a compatibility wrapper using default limits.

## 9.4. Limit Behavior

When a limit is reached, an extractor must choose one of two behaviors:

### Hard stop

Return an error:

```rust
ExtractErrorCategory::TooLarge
```

Use this when continuing would be unsafe or misleading.

### Partial output with warning

Return partial segments plus warning:

```rust
ExtractWarning::SizeLimitReached
```

Use this when partial extraction is still useful and honest.

## 9.5. Required Checks

Every extractor must check:

- input file size before reading;
- extracted character count while producing segments;
- segment count;
- format-specific limits.

Format-specific requirements:

| Format | Required limit |
|---|---|
| text | max file bytes, max extracted chars |
| Markdown | max file bytes, max extracted chars, max segments |
| HTML | max HTML bytes, max extracted chars, max segments |
| DOCX | max ZIP entry bytes, max XML bytes, max extracted chars |
| PDF | max file bytes, max pages, max extracted chars |

---

## 10. Structured Extraction Warnings

## 10.1. Problem

The current output does not provide a warning channel. Without warnings, the app cannot distinguish:

```text
file fully prepared
```

from:

```text
file partially prepared, but some pages or parts were skipped
```

This matters for trust and UX.

## 10.2. Required Type

Add:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractWarning {
    SomeContentSkipped {
        reason: String,
    },
    SomePagesUnreadable {
        pages: Vec<u32>,
    },
    PossiblyScannedPdf,
    SizeLimitReached {
        limit_name: String,
    },
    EncodingUnsupported,
    UnsupportedDocumentPart {
        part: String,
    },
    ApproximateLocationOnly,
    MalformedContentRecovered,
}
```

If string payloads are considered too loose, replace them with typed enums.

## 10.3. Output Change

Change:

```rust
pub struct ExtractOutput {
    pub extractor_name: String,
    pub extractor_version: String,
    pub normalization_version: String,
    pub segments: Vec<ExtractedSegment>,
    pub char_count: u64,
}
```

to:

```rust
pub struct ExtractOutput {
    pub extractor_name: String,
    pub extractor_version: String,
    pub normalization_version: String,
    pub segments: Vec<ExtractedSegment>,
    pub char_count: u64,
    pub warnings: Vec<ExtractWarning>,
}
```

## 10.4. UI Mapping

`orbok-ui` should not show raw warning names.

Example mapping:

| Warning | User-facing message |
|---|---|
| SomePagesUnreadable | Some pages could not be prepared. |
| PossiblyScannedPdf | This PDF may contain images instead of selectable text. |
| SizeLimitReached | Only part of this large file was prepared. |
| UnsupportedDocumentPart | Some document parts were skipped. |
| EncodingUnsupported | This file uses text encoding orbok could not read. |

Default UI should show only important warnings. Detailed warnings can live in Advanced view.

---

## 11. Panic Isolation

## 11.1. Problem

Parser crates can panic on malformed inputs. Comments in the PDF extractor suggest panic isolation, but review did not find an obvious wrapper in `orbok-extract`.

Even if a worker layer catches panics elsewhere, extraction safety should be explicit and testable.

## 11.2. Required Safe Wrapper

Add registry-level safe extraction:

```rust
pub fn extract_safely(
    &self,
    path: &ValidatedPath,
    context: &ExtractContext,
) -> OrbokResult<ExtractOutput>
```

Implementation should catch panics around extractor invocation:

```rust
let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    extractor.extract_with_context(path, context)
}));
```

On panic:

```rust
Err(OrbokError::extraction(
    ExtractErrorCategory::ParserPanic,
    "extractor failed while reading this file",
))
```

The user-facing layer should say:

```text
This file could not be prepared.
```

not:

```text
panic
```

## 11.3. Isolation Scope

Panic isolation should wrap:

- PDF extraction;
- DOCX extraction;
- HTML extraction if parser complexity grows;
- any future external extractor;
- plugin extractor calls.

## 11.4. Test Requirement

Add a test extractor that intentionally panics.

Expected result:

```text
extract_safely returns typed error
process does not crash
other files can still be extracted
```

---

## 12. Location Semantics

## 12.1. Problem

`ExtractedSegment` currently uses line-like fields. For different formats, those fields may mean different things:

- text: lines;
- Markdown: lines;
- PDF: pages;
- DOCX: paragraphs;
- HTML: approximate blocks.

This is workable internally but risky for downstream display.

The UI should not accidentally display:

```text
line 7
```

when the value actually means:

```text
page 7
```

## 12.2. Minimal Required Fix

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationKind {
    Lines,
    Pages,
    Paragraphs,
    Blocks,
    Unknown,
}
```

Then add:

```rust
pub location_kind: LocationKind
```

to `ExtractedSegment`.

## 12.3. Mapping

| Format | LocationKind |
|---|---|
| text | Lines |
| Markdown | Lines |
| PDF | Pages |
| DOCX | Paragraphs |
| HTML | Blocks |
| unknown/fallback | Unknown |

## 12.4. Future Richer Model

A later design may replace this with:

```rust
pub enum SourceLocation {
    Lines { start: u32, end: u32 },
    Pages { start: u32, end: u32 },
    Paragraphs { start: u32, end: u32 },
    Blocks { start: u32, end: u32 },
    Unknown,
}
```

This RFC does not require that larger migration immediately.

---

## 13. Consistent Error Mapping

## 13.1. Problem

Some extractors explicitly map read errors, while others use direct `?` conversions. This may produce inconsistent categories.

## 13.2. Required Error Categories

Define or standardize categories similar to:

```rust
pub enum ExtractErrorCategory {
    SourceMissing,
    PermissionDenied,
    UnsupportedFormat,
    EncodingError,
    ParserError,
    ParserPanic,
    EncryptedDocument,
    TooLarge,
    IoError,
    Internal,
}
```

## 13.3. Required Mapping

| Condition | Category |
|---|---|
| file not found | SourceMissing |
| permission denied | PermissionDenied |
| invalid UTF-8 when strict | EncodingError |
| malformed PDF/XML/ZIP | ParserError |
| parser panic | ParserPanic |
| encrypted PDF | EncryptedDocument |
| file or part too large | TooLarge |
| unsupported extension | UnsupportedFormat |

## 13.4. User-Facing Mapping

Default UI should show:

| Category | Message |
|---|---|
| SourceMissing | File not found. It may have been moved. |
| PermissionDenied | orbok cannot open this file. |
| EncodingError | This text file could not be read. |
| ParserError | This file could not be prepared. |
| ParserPanic | This file could not be prepared. |
| EncryptedDocument | This file is locked. |
| TooLarge | This file is too large to prepare safely. |
| UnsupportedFormat | This file type is not supported yet. |

---

## 14. Crate Boundary Cleanup

## 14.1. Problem

`orbok-extract` currently includes chunking and depends on `orbok-db` through chunk output types.

That creates a dependency direction concern:

```text
orbok-extract → orbok-db
```

Extraction should ideally not depend on the persistence layer.

## 14.2. Accepted Direction

Remove direct `orbok-db` dependency from `orbok-extract`.

## 14.3. Option A: Move Chunking Out

Move chunking to:

```text
orbok-pipeline
```

or:

```text
orbok-chunk
```

Architecture:

```text
orbok-extract
  ↓ ExtractOutput
orbok-chunk
  ↓ ChunkSpec
orbok-db
```

This is the cleanest long-term direction.

## 14.4. Option B: Keep Chunking but Use Neutral Type

Keep chunking in `orbok-extract`, but define a neutral chunk type:

```rust
pub struct ExtractedChunk {
    pub chunk_kind: String,
    pub chunk_ordinal: u32,
    pub heading_path: Option<String>,
    pub title: Option<String>,
    pub normalized_text: String,
    pub location_kind: LocationKind,
    pub location_start: u32,
    pub location_end: u32,
    pub byte_start: Option<u64>,
    pub byte_end: Option<u64>,
    pub location_quality: LocationQuality,
    pub parent_idx: Option<usize>,
}
```

Then `orbok-db` maps:

```text
ExtractedChunk → db::ChunkSpec
```

## 14.5. Recommendation

Use Option B as P0 if it is lower risk.

Use Option A later if the team wants a cleaner split.

## 14.6. Boundary Rule

After this RFC:

```text
orbok-extract must not depend on orbok-db.
```

---

## 15. Test Boundary Cleanup

## 15.1. Problem

Some tests inside `orbok-extract` reference model/embedding concerns.

This is not necessarily runtime coupling, but it makes the crate responsibility less clear.

## 15.2. Required Direction

Move non-extraction tests to the owning crates:

| Test concern | Target |
|---|---|
| model integrity | `orbok-models` |
| embedding configuration | `orbok-embed` |
| model setup integration | `orbok-app` or integration tests |
| extraction output | `orbok-extract` |

## 15.3. Rule

`orbok-extract` tests should focus on:

- extractors;
- normalization;
- segment locations;
- warnings;
- limits;
- registry;
- chunking only if chunking remains in crate.

---

## 16. Format-Specific Requirements

## 16.1. Text Extractor

Required:

- file size limit before reading;
- strict UTF-8 behavior documented;
- encoding error category;
- no panic on invalid bytes;
- line location kind.

P1 consideration:

- optional encoding detection if real user files need Shift-JIS or legacy Japanese encodings.

## 16.2. Markdown Extractor

Required:

- file size limit;
- segment count limit;
- heading/code-fence tests remain;
- line location kind;
- warning if size limit truncates.

P1:

- setext heading support;
- frontmatter handling;
- table/list structure tests.

## 16.3. HTML Extractor

Required:

- file size limit;
- extracted char limit;
- block location kind;
- warning for malformed/recovered content if detectable;
- consistent I/O error mapping.

P1:

- better entity decoding;
- parser quality fixtures.

## 16.4. DOCX Extractor

Required:

- ZIP entry size limit;
- XML size limit;
- paragraph location kind;
- warning for unsupported skipped document parts;
- parser error category for malformed ZIP/XML;
- no panic on malformed ZIP.

P1:

- headers, footers, footnotes, comments;
- better entity handling;
- table text tests.

## 16.5. PDF Extractor

Required:

- file size limit;
- page count limit;
- extracted char limit;
- page location kind;
- warnings for unreadable pages;
- warning for possibly scanned PDF;
- parser panic isolation;
- encrypted PDF category.

P1:

- stronger fixtures with known extractable text;
- Japanese PDF quality review;
- alternative backend evaluation only if needed.

---

## 17. Plugin Scope

`plugin.rs` currently appears to be future scaffolding.

This RFC does not expand plugin support.

Allowed:

- keep manifest structures;
- keep compile-time scaffolding;
- keep future notes.

Not allowed in this RFC:

- dynamic loading;
- external process plugins;
- WASM plugin runtime;
- plugin marketplace;
- security sandbox design.

If desired, gate plugin scaffolding behind a feature:

```toml
plugin-manifest = []
```

But this is optional.

---

## 18. Compatibility and Migration

## 18.1. ExtractOutput Migration

Adding `warnings` can be backward-compatible if initialized as empty:

```rust
warnings: Vec::new()
```

## 18.2. LocationKind Migration

Existing segments can map:

```text
TextExtractor      → Lines
MarkdownExtractor  → Lines
PdfExtractor       → Pages
DocxExtractor      → Paragraphs
HtmlExtractor      → Blocks
```

## 18.3. Chunking Migration

If introducing `ExtractedChunk`, implement conversion:

```rust
impl From<ExtractedChunk> for orbok_db::repo::ChunkSpec
```

outside `orbok-extract`, likely in pipeline or DB adapter code.

## 18.4. Trait Migration

If adding context to `DocumentExtractor` is too disruptive:

1. add a new method with default implementation;
2. migrate built-in extractors;
3. later remove old method.

Example:

```rust
fn extract_with_context(
    &self,
    path: &ValidatedPath,
    context: &ExtractContext,
) -> OrbokResult<ExtractOutput> {
    self.extract(path)
}
```

Then migrate extractors one by one.

---

## 19. Implementation Priority

## 19.1. P0

Implement before broader release hardening:

1. `ExtractLimits`;
2. `ExtractWarning`;
3. `warnings` field in `ExtractOutput`;
4. panic isolation wrapper;
5. consistent I/O and parser error mapping;
6. PDF page/text limits;
7. DOCX ZIP/XML limits;
8. HTML/text/Markdown file limits;
9. `LocationKind`;
10. remove direct `orbok-db` dependency or introduce neutral chunk type;
11. move non-extraction tests out.

## 19.2. P1

Implement after P0:

1. stronger PDF fixtures;
2. malformed DOCX/HTML fixtures;
3. better DOCX entity/document-part handling;
4. optional encoding detection decision;
5. warning UI mapping in app layer;
6. richer integration tests.

## 19.3. P2

Future only:

1. spreadsheet extraction;
2. OCR detection and pipeline;
3. richer PDF backend evaluation;
4. dynamic plugin system;
5. full `SourceLocation` enum migration.

---

## 20. Test Plan

## 20.1. Limit Tests

- text file over max size returns TooLarge or partial warning;
- Markdown segment limit is enforced;
- HTML byte limit is enforced;
- DOCX XML entry limit is enforced;
- PDF page limit is enforced;
- extracted char limit is enforced.

## 20.2. Warning Tests

- PDF with unreadable page emits warning;
- scanned/no-text PDF emits `PossiblyScannedPdf`;
- truncated large file emits `SizeLimitReached`;
- DOCX skipped unsupported parts emit warning;
- approximate location emits warning where appropriate.

## 20.3. Panic Isolation Tests

- panic extractor returns ParserPanic;
- panic does not abort process;
- registry continues to work after panic;
- worker can proceed to next file.

## 20.4. Error Mapping Tests

- missing file → SourceMissing;
- permission denied → PermissionDenied where platform allows;
- invalid UTF-8 → EncodingError;
- malformed PDF → ParserError or ParserPanic;
- encrypted PDF → EncryptedDocument;
- unsupported extension → UnsupportedFormat.

## 20.5. Location Tests

- text segments use Lines;
- Markdown segments use Lines;
- PDF segments use Pages;
- DOCX segments use Paragraphs;
- HTML segments use Blocks;
- UI/display adapter does not call pages “lines.”

## 20.6. Boundary Tests

- `orbok-extract` does not depend on `orbok-db`;
- model tests are not inside `orbok-extract`;
- extraction tests do not require model/embedding crates.

---

## 21. Acceptance Criteria

This RFC is accepted when:

1. `orbok-extract` has resource limits.
2. All built-in extractors observe relevant limits.
3. `ExtractOutput` includes structured warnings.
4. PDF unreadable pages and scanned/no-text PDFs can be represented as warnings.
5. Extractor panics are caught and converted to typed errors.
6. Errors are consistently mapped across extractors.
7. Segment location semantics distinguish lines, pages, paragraphs, and blocks.
8. `orbok-extract` no longer depends directly on `orbok-db`, or a clear neutral chunk type is introduced.
9. Non-extraction tests are moved out of `orbok-extract`.
10. Malformed and oversized file tests exist.
11. The app can continue indexing other files when one extraction fails.
12. No broad rewrite of the extraction architecture is introduced.

---

## 22. Explicit Rejections

This RFC rejects:

```text
adopting findtext-* directly
rewriting orbok-extract from scratch
moving keyword search into orbok-extract
exposing parser details in the default UI
adding plugin loading now
adding OCR now
changing GUI framework as part of extraction hardening
```

---

## 23. Final Decision

`orbok-extract v0.9` is already the right architectural foundation.

The next step is not extraction-boundary design.

The next step is:

```text
production hardening
resource limits
warnings
panic isolation
location clarity
crate dependency cleanup
test gates
```

This RFC defines that focused path.
