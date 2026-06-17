//! Adaptive chunker (RFC-006 §7–§9).
//!
//! Strategy (RFC-006 §22 decision):
//! - **Document chunk** (parent, ordinal 0): the whole document, title
//!   = filename, section spans the full file. Used for passage-level
//!   reranking context.
//! - **Section chunks** (children): for Markdown, one child per heading
//!   section; for plain text, one child per paragraph group; for other
//!   types, one child (the whole document).
//! - **Fallback split**: paragraph groups that exceed `MAX_CHARS` are
//!   split into overlapping windows (RFC-006 §8.3 token-window fallback).
//!
//! Location quality: Markdown and plain text produce exact line ranges;
//! other types produce approximate (RFC-006 §16).
//!
//! The output is pure data — no I/O, no DB access, easily testable.

use crate::types::{ExtractOutput, SegmentKind};
use orbok_db::repo::ChunkSpec;

/// Characters per fallback window (approximate token proxy).
const MAX_CHARS: usize = 1200;
/// Overlap between fallback windows.
const OVERLAP_CHARS: usize = 120;

/// Chunk a single-file extraction result into a flat list of
/// [`ChunkSpec`]s ready for [`ChunkRepository::insert_bundle`].
///
/// Index 0 is always the document-level parent chunk.
/// Indices 1..N are the child (leaf) chunks used for retrieval.
pub fn chunk(output: &ExtractOutput, file_display_name: &str) -> Vec<ChunkSpec> {
    if output.segments.is_empty() {
        return vec![empty_document_chunk(file_display_name)];
    }
    let all_text: String = output
        .segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let first_line = output.segments.first().map(|s| s.line_start).unwrap_or(1);
    let last_line = output.segments.last().map(|s| s.line_end).unwrap_or(1);

    // Parent chunk (ordinal 0).
    let parent_title = file_display_name.to_string();
    let parent_heading = output.segments.iter().find_map(|s| {
        if s.kind == SegmentKind::Heading {
            Some(s.text.trim().to_string())
        } else {
            None
        }
    });
    let parent = ChunkSpec {
        chunk_kind: "document",
        chunk_ordinal: 0,
        heading_path: parent_heading,
        title: Some(parent_title),
        normalized_text: all_text,
        line_start: first_line,
        line_end: last_line,
        byte_start: None,
        byte_end: None,
        location_quality: "exact",
        parent_idx: None,
    };

    let mut specs = vec![parent];

    // Child chunks: strategy by content.
    let has_markdown_structure = output
        .segments
        .iter()
        .any(|s| s.kind == SegmentKind::Heading);

    if has_markdown_structure {
        append_markdown_sections(output, &mut specs);
    } else {
        append_paragraph_chunks(output, &mut specs);
    }

    // Assign sequential ordinals (parent is 0, children start at 1).
    for (i, spec) in specs.iter_mut().enumerate() {
        spec.chunk_ordinal = i as u32;
    }
    specs
}

/// One child chunk per heading section in a Markdown document.
fn append_markdown_sections(output: &ExtractOutput, specs: &mut Vec<ChunkSpec>) {
    struct Section {
        heading: String,
        heading_path: String,
        segments: Vec<usize>,
    }

    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::new();
    let mut heading_trail: Vec<String> = Vec::new();

    for (i, seg) in output.segments.iter().enumerate() {
        if seg.kind == SegmentKind::Heading {
            let level = seg
                .heading_path
                .as_deref()
                .map(|h| h.matches('>').count() + 1)
                .unwrap_or(1);
            heading_trail.truncate(level.saturating_sub(1));
            heading_trail.push(seg.text.trim().to_string());
            current_heading = seg.text.trim().to_string();
            let path = heading_trail.join(" > ");
            sections.push(Section {
                heading: current_heading.clone(),
                heading_path: path,
                segments: vec![i],
            });
        } else if let Some(section) = sections.last_mut() {
            section.segments.push(i);
        } else {
            // Content before the first heading.
            sections.push(Section {
                heading: current_heading.clone(),
                heading_path: String::new(),
                segments: vec![i],
            });
        }
    }

    for section in sections {
        if section.segments.is_empty() {
            continue;
        }
        let first = &output.segments[*section.segments.first().unwrap()];
        let last = &output.segments[*section.segments.last().unwrap()];
        let text: String = section
            .segments
            .iter()
            .map(|&i| output.segments[i].text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if text.trim().is_empty() {
            continue;
        }
        // Long sections get split further via the fallback mechanism.
        if text.len() > MAX_CHARS {
            append_text_windows(&text, first.line_start, last.line_end,
                                Some(section.heading_path.clone()), specs);
        } else {
            specs.push(ChunkSpec {
                chunk_kind: "section",
                chunk_ordinal: 0,
                heading_path: Some(section.heading_path),
                title: Some(section.heading),
                normalized_text: text,
                line_start: first.line_start,
                line_end: last.line_end,
                byte_start: None,
                byte_end: None,
                location_quality: "exact",
                parent_idx: Some(0),
            });
        }
    }
}

/// One child chunk per paragraph (or paragraph group within limit).
fn append_paragraph_chunks(output: &ExtractOutput, specs: &mut Vec<ChunkSpec>) {
    let mut buf = String::new();
    let mut buf_start = 0u32;
    let mut buf_end = 0u32;

    let flush = |buf: &mut String, start: u32, end: u32, specs: &mut Vec<ChunkSpec>| {
        let text = buf.trim().to_string();
        if text.is_empty() {
            buf.clear();
            return;
        }
        if text.len() > MAX_CHARS {
            append_text_windows(&text, start, end, None, specs);
        } else {
            specs.push(ChunkSpec {
                chunk_kind: "paragraph",
                chunk_ordinal: 0,
                heading_path: None,
                title: None,
                normalized_text: text,
                line_start: start,
                line_end: end,
                byte_start: None,
                byte_end: None,
                location_quality: "exact",
                parent_idx: Some(0),
            });
        }
        buf.clear();
    };

    for seg in &output.segments {
        if buf_start == 0 {
            buf_start = seg.line_start;
        }
        buf_end = seg.line_end;
        buf.push_str(&seg.text);
        buf.push('\n');
        if buf.len() >= MAX_CHARS {
            flush(&mut buf, buf_start, buf_end, specs);
            buf_start = seg.line_end + 1;
        }
    }
    if !buf.trim().is_empty() {
        flush(&mut buf, buf_start, buf_end, specs);
    }
}

/// Split a long text into overlapping fallback windows (RFC-006 §8.3).
fn append_text_windows(
    text: &str,
    line_start: u32,
    line_end: u32,
    heading_path: Option<String>,
    specs: &mut Vec<ChunkSpec>,
) {
    let chars: Vec<char> = text.chars().collect();
    let total_lines = (line_end - line_start).max(1);
    let mut start = 0usize;
    let mut window_idx = 0u32;
    while start < chars.len() {
        let end = (start + MAX_CHARS).min(chars.len());
        let window_text: String = chars[start..end].iter().collect();
        // Approximate line range for this window.
        let frac_start = start as f64 / chars.len() as f64;
        let frac_end = end as f64 / chars.len() as f64;
        let wl_start = line_start + (total_lines as f64 * frac_start) as u32;
        let wl_end = line_start + (total_lines as f64 * frac_end) as u32;
        specs.push(ChunkSpec {
            chunk_kind: "fallback",
            chunk_ordinal: 0,
            heading_path: heading_path.clone(),
            title: None,
            normalized_text: window_text.trim().to_string(),
            line_start: wl_start,
            line_end: wl_end,
            byte_start: None,
            byte_end: None,
            location_quality: "approximate",
            parent_idx: Some(0),
        });
        if end == chars.len() {
            break;
        }
        start = end.saturating_sub(OVERLAP_CHARS);
        window_idx += 1;
        let _ = window_idx;
    }
}

fn empty_document_chunk(name: &str) -> ChunkSpec {
    ChunkSpec {
        chunk_kind: "document",
        chunk_ordinal: 0,
        heading_path: None,
        title: Some(name.to_string()),
        normalized_text: String::new(),
        line_start: 1,
        line_end: 1,
        byte_start: None,
        byte_end: None,
        location_quality: "unknown",
        parent_idx: None,
    }
}
