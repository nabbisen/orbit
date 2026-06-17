//! Markdown extractor, `markdown-v1` (RFC-005 §5/§8: heading-aware,
//! fence-aware, exact line locations for FR-061 "open at location").
//!
//! Line-oriented by design: every segment maps to exact source lines so
//! search results can highlight the original file region. ATX headings
//! (`#`–`######`) maintain the heading path; fenced code blocks become
//! [`SegmentKind::CodeBlock`]; everything else groups into paragraphs.

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
    read_error_category,
};
use orbit_core::{ErrorCategory, OrbitError, OrbitResult, versions::NORMALIZATION_VERSION};
use orbit_fs::ValidatedPath;

pub struct MarkdownExtractor;

impl DocumentExtractor for MarkdownExtractor {
    fn name(&self) -> &'static str {
        "markdown"
    }

    fn version(&self) -> &'static str {
        "markdown-v1"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["md", "markdown"]
    }

    fn extract(&self, path: &ValidatedPath) -> OrbitResult<ExtractOutput> {
        let bytes = std::fs::read(&path.canonical).map_err(|e| OrbitError::Extraction {
            category: read_error_category(&e),
            message: e.to_string(),
        })?;
        let raw = String::from_utf8(bytes).map_err(|_| OrbitError::Extraction {
            category: ErrorCategory::EncodingError,
            message: "file is not valid UTF-8".into(),
        })?;
        let normalized = normalize_document(&raw);
        let segments = parse_markdown(&normalized);
        Ok(ExtractOutput {
            extractor_name: self.name().into(),
            extractor_version: self.version().into(),
            normalization_version: NORMALIZATION_VERSION.into(),
            char_count: normalized.chars().count() as u64,
            segments,
        })
    }
}

struct HeadingStack(Vec<(u8, String)>);

impl HeadingStack {
    fn push(&mut self, level: u8, title: &str) {
        self.0.retain(|(l, _)| *l < level);
        self.0.push((level, title.to_string()));
    }

    fn path(&self) -> Option<String> {
        if self.0.is_empty() {
            None
        } else {
            Some(
                self.0
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > "),
            )
        }
    }
}

fn parse_markdown(normalized: &str) -> Vec<ExtractedSegment> {
    let lines: Vec<&str> = normalized.lines().collect();
    let mut segments = Vec::new();
    let mut headings = HeadingStack(Vec::new());
    let mut paragraph: Vec<&str> = Vec::new();
    let mut paragraph_start = 0u32;
    let mut idx = 0usize;

    macro_rules! flush_paragraph {
        ($end:expr) => {
            if !paragraph.is_empty() {
                segments.push(ExtractedSegment {
                    kind: SegmentKind::Paragraph,
                    text: paragraph.join("\n"),
                    line_start: paragraph_start,
                    line_end: $end,
                    heading_path: headings.path(),
                    location_quality: LocationQuality::Exact,
                });
                paragraph.clear();
            }
        };
    }

    while idx < lines.len() {
        let line = lines[idx];
        let line_no = idx as u32 + 1;

        // ATX heading.
        if let Some((level, title)) = parse_atx_heading(line) {
            flush_paragraph!(line_no - 1);
            headings.push(level, title);
            segments.push(ExtractedSegment {
                kind: SegmentKind::Heading,
                text: title.to_string(),
                line_start: line_no,
                line_end: line_no,
                heading_path: headings.path(),
                location_quality: LocationQuality::Exact,
            });
            idx += 1;
            continue;
        }

        // Fenced code block.
        if let Some(fence) = parse_fence(line) {
            flush_paragraph!(line_no - 1);
            let start = line_no;
            let mut body = Vec::new();
            idx += 1;
            while idx < lines.len() && parse_fence(lines[idx]) != Some(fence) {
                body.push(lines[idx]);
                idx += 1;
            }
            let end = (idx as u32) + 1; // closing fence (or EOF line)
            idx += 1; // skip the closing fence when present
            segments.push(ExtractedSegment {
                kind: SegmentKind::CodeBlock,
                text: body.join("\n"),
                line_start: start,
                line_end: end.min(lines.len() as u32),
                heading_path: headings.path(),
                location_quality: LocationQuality::Exact,
            });
            continue;
        }

        // Blank line ends a paragraph.
        if line.trim().is_empty() {
            flush_paragraph!(line_no - 1);
            idx += 1;
            continue;
        }

        if paragraph.is_empty() {
            paragraph_start = line_no;
        }
        paragraph.push(line);
        idx += 1;
    }
    let last = lines.len() as u32;
    flush_paragraph!(last);
    segments
}

/// `#`–`######` followed by a space → (level, title).
fn parse_atx_heading(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&hashes) {
        let rest = &trimmed[hashes..];
        if let Some(title) = rest.strip_prefix(' ') {
            let title = title.trim().trim_end_matches('#').trim_end();
            if !title.is_empty() {
                return Some((hashes as u8, title));
            }
        }
    }
    None
}

/// Code fence marker: three-or-more backticks or tildes. Returns the
/// fence character so open/close pairs match.
fn parse_fence(line: &str) -> Option<char> {
    let trimmed = line.trim_start();
    for fence_char in ['`', '~'] {
        let count = trimmed.chars().take_while(|c| *c == fence_char).count();
        if count >= 3 {
            return Some(fence_char);
        }
    }
    None
}
