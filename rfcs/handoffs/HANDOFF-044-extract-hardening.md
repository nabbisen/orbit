# Implementation Handoff — RFC-044: orbok-extract Production Hardening and Boundary Cleanup

**Project:** orbok  
**RFC:** 044  
**Implementation theme:** harden existing `orbok-extract`, not rewrite it  
**Primary owners:** extraction crate/pipeline/db boundary/tests

> **Implementation rule:** Keep default UI plain and safe for non-technical users. Advanced details may exist, but the normal path must not expose internal terms such as `index`, `cache`, `query`, `vector`, `embedding`, `source`, `schema`, or raw parser/network errors.

> **Project name rule:** Use **orbok** in all new UI copy, docs, diagnostics, and tests. Treat **orbit** as historical only.

## 1. Outcome

Harden `orbok-extract v0.9` for production without redefining the architecture.

Implement:

```text
limits
warnings
panic isolation
location clarity
consistent errors
crate boundary cleanup
test gates
```

## 2. Scope

### In scope

- `ExtractLimits`.
- `ExtractWarning` and `warnings` in output.
- Panic isolation wrapper.
- `LocationKind` or equivalent.
- Consistent extractor error mapping.
- Remove or neutralize `orbok-db` dependency from `orbok-extract`.
- Move non-extraction tests out.
- Add malformed/large file tests.

### Out of scope

- Replacing `lopdf`.
- OCR.
- XLSX extraction.
- Dynamic plugins.
- Adopting `findtext-*`.
- Moving keyword search into extraction.

## 3. Implementation Boundaries

Recommended modules:

```text
crates/orbok-extract/src/types.rs
crates/orbok-extract/src/registry.rs
crates/orbok-extract/src/text.rs
crates/orbok-extract/src/markdown.rs
crates/orbok-extract/src/html.rs
crates/orbok-extract/src/docx.rs
crates/orbok-extract/src/pdf.rs
crates/orbok-extract/src/chunker.rs
crates/orbok-pipeline/src/chunk_adapter.rs
```

## 4. Data / API Changes

Add:

```rust
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

Add:

```rust
pub enum ExtractWarning {
    SomeContentSkipped { reason: String },
    SomePagesUnreadable { pages: Vec<u32> },
    PossiblyScannedPdf,
    SizeLimitReached { limit_name: String },
    EncodingUnsupported,
    UnsupportedDocumentPart { part: String },
    ApproximateLocationOnly,
    MalformedContentRecovered,
}
```

Add:

```rust
pub enum LocationKind {
    Lines,
    Pages,
    Paragraphs,
    Blocks,
    Unknown,
}
```

## 5. PR Plan

### PR-035-1 — Output warnings and limits foundation

Tasks:

- Add `ExtractLimits` with defaults.
- Add `ExtractWarning`.
- Add `warnings` to `ExtractOutput`.
- Update constructors/tests.

Acceptance:

- Existing extractors compile with empty warnings.
- Limits can be passed or defaulted.

### PR-035-2 — Built-in extractor limits

Tasks:

- Text/Markdown/HTML file size limits.
- DOCX ZIP/XML limits.
- PDF file/page/text limits.
- Segment count and extracted char limits.

Acceptance:

- Oversized files do not cause unbounded reads.
- Limit tests pass.

### PR-035-3 — Panic isolation and error mapping

Tasks:

- Add `extract_safely` registry wrapper.
- Catch extractor panics.
- Standardize categories: SourceMissing, PermissionDenied, EncodingError, ParserError, ParserPanic, EncryptedDocument, TooLarge, UnsupportedFormat.

Acceptance:

- Panic test extractor returns typed error.
- Malformed file does not crash worker.

### PR-035-4 — Location semantics

Tasks:

- Add `LocationKind` to segments.
- Map text/Markdown to Lines.
- Map PDF to Pages.
- Map DOCX to Paragraphs.
- Map HTML to Blocks.

Acceptance:

- UI adapters can avoid saying “line” for PDF pages.

### PR-035-5 — Boundary cleanup

Tasks:

- Remove direct `orbok-db` dependency or introduce neutral chunk type.
- Move DB mapping to pipeline/db adapter.
- Move model/embedding tests out of `orbok-extract`.

Acceptance:

- `orbok-extract` does not depend on `orbok-db`.
- Extraction tests do not require model crates.

## 6. Acceptance Criteria

- Resource limits exist and are observed.
- Warnings exist and are populated for partial extraction.
- Panics are isolated.
- Errors are consistently categorized.
- Location kind is explicit.
- `orbok-extract` boundary is cleaner.
- Malformed and large file tests exist.
- One bad file does not block other files.

## 7. QA Checklist

- Large text file.
- Invalid UTF-8 text.
- Large Markdown file.
- Malformed HTML.
- DOCX with large XML entry.
- Malformed DOCX.
- PDF with many pages.
- Encrypted PDF.
- No-text/scanned PDF.
- Panic extractor test.
- Confirm UI receives warnings and not raw parser messages.
