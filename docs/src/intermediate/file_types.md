# Supported File Types

## v0.x supported (initial)

| Type | Extensions | Location quality |
|---|---|---|
| Plain text | `.txt`, `.log` | Exact line/byte |
| Markdown | `.md`, `.markdown` | Heading/line |
| HTML | `.html`, `.htm` | Approximate |
| PDF | `.pdf` | Page-level |
| DOCX | `.docx` | Paragraph-level |
| CSV | `.csv` | Row-level |
| Source code | `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.c`, `.cpp`, `.sql`, and more | Line-level |
| Config | `.toml`, `.yaml`, `.yml`, `.json`, `.xml` | Line-level |

## Unsupported (planned)

XLSX, PPTX, email archives, browser bookmarks, OCR images — see the
roadmap for planned support.

## Unsupported (skipped silently)

Binary files, encrypted files, and files exceeding the configured
maximum size are cataloged as `unsupported` and skipped during indexing.
