//! HTML text extractor.
//!
//! Strips HTML tags with a simple state-machine parser and preserves
//! visible text content. Block-level elements (`p`, `div`, `h1`–`h6`,
//! `li`, `td`, `th`, `br`) produce paragraph boundaries.
//! `<h1>`–`<h6>` headings populate `heading_path`.
//!
//! Security: no JavaScript execution, no external resource loading,
//! no DOM construction. Pure text extraction only (RFC-015 §15).

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
};
use orbok_core::{OrbokResult, versions::NORMALIZATION_VERSION};
use orbok_fs::ValidatedPath;

const EXTRACTOR_NAME: &str = "html";
const EXTRACTOR_VERSION: &str = "v1";

pub struct HtmlExtractor;

impl DocumentExtractor for HtmlExtractor {
    fn name(&self) -> &'static str {
        EXTRACTOR_NAME
    }
    fn version(&self) -> &'static str {
        EXTRACTOR_VERSION
    }
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["html", "htm"]
    }

    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        let content = std::fs::read_to_string(&path.canonical)?;
        let blocks = extract_blocks(&content);
        let mut segments = Vec::new();
        let mut total_chars = 0u64;
        let mut line = 1u32;

        for block in &blocks {
            let norm = normalize_document(&block.text);
            if norm.trim().is_empty() {
                line += 1;
                continue;
            }
            total_chars += norm.len() as u64;
            segments.push(ExtractedSegment {
                kind: block.kind,
                text: norm,
                line_start: line,
                line_end: line,
                heading_path: block.heading.clone(),
                location_quality: LocationQuality::Approximate,
            });
            line += 1;
        }

        Ok(ExtractOutput {
            extractor_name: EXTRACTOR_NAME.to_string(),
            extractor_version: EXTRACTOR_VERSION.to_string(),
            normalization_version: NORMALIZATION_VERSION.to_string(),
            segments,
            char_count: total_chars,
        })
    }
}

struct Block {
    text: String,
    kind: SegmentKind,
    heading: Option<String>,
}

/// Block-level elements that cause paragraph breaks.
const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "aside",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "dt",
    "dd",
    "td",
    "th",
    "caption",
    "blockquote",
    "pre",
    "br",
];
const HEADING_TAGS: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];
/// Tags whose content should be suppressed entirely.
const SKIP_TAGS: &[&str] = &["script", "style", "head", "noscript", "template"];

fn extract_blocks(html: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_text = String::new();
    let mut current_heading: Option<String> = None;
    let mut heading_trail: Vec<String> = Vec::new();
    let mut skip_depth: usize = 0;
    let mut skip_tag = "";
    let mut pos = 0;
    let chars: Vec<char> = html.chars().collect();
    let n = chars.len();

    let flush = |text: &mut String, heading: &Option<String>, blocks: &mut Vec<Block>| {
        let t = text.trim().to_string();
        if !t.is_empty() {
            blocks.push(Block {
                text: t,
                kind: SegmentKind::Paragraph,
                heading: heading.clone(),
            });
        }
        text.clear();
    };

    while pos < n {
        if chars[pos] == '<' {
            // Collect tag
            let mut tag_end = pos + 1;
            while tag_end < n && chars[tag_end] != '>' {
                tag_end += 1;
            }
            let tag_str: String = chars[pos..tag_end.min(n)].iter().collect();
            let is_close = tag_str.starts_with("</");
            let tag_name = tag_str
                .trim_start_matches('<')
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();

            if skip_depth > 0 {
                if is_close && tag_name == skip_tag {
                    skip_depth -= 1;
                } else if !is_close && tag_name == skip_tag {
                    skip_depth += 1;
                }
            } else if !is_close && SKIP_TAGS.contains(&tag_name.as_str()) {
                skip_tag = SKIP_TAGS
                    .iter()
                    .find(|&&t| t == tag_name.as_str())
                    .copied()
                    .unwrap_or("");
                skip_depth = 1;
                flush(&mut current_text, &current_heading, &mut blocks);
            } else if is_close && HEADING_TAGS.contains(&tag_name.as_str()) {
                // Closing a heading: record it and update heading_path context.
                let heading_text = current_text.trim().to_string();
                if !heading_text.is_empty() {
                    let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                    heading_trail.truncate(level.saturating_sub(1));
                    heading_trail.push(heading_text.clone());
                    current_heading = Some(heading_trail.join(" > "));
                    blocks.push(Block {
                        text: heading_text,
                        kind: SegmentKind::Heading,
                        heading: current_heading.clone(),
                    });
                    current_text.clear();
                }
            } else if BLOCK_TAGS.contains(&tag_name.as_str()) {
                flush(&mut current_text, &current_heading, &mut blocks);
                if !is_close && HEADING_TAGS.contains(&tag_name.as_str()) {
                    let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                    heading_trail.truncate(level.saturating_sub(1));
                }
            }
            pos = tag_end + 1;
        } else if skip_depth == 0 {
            // Decode common HTML entities inline.
            let c = chars[pos];
            if c == '&' {
                let semi = chars[pos..].iter().position(|&x| x == ';').map(|p| pos + p);
                if let Some(end) = semi {
                    let entity: String = chars[pos..=end].iter().collect();
                    match entity.as_str() {
                        "&amp;" => current_text.push('&'),
                        "&lt;" => current_text.push('<'),
                        "&gt;" => current_text.push('>'),
                        "&quot;" | "&ldquo;" | "&rdquo;" => current_text.push('"'),
                        "&apos;" | "&lsquo;" | "&rsquo;" => current_text.push('\''),
                        "&nbsp;" | "&#160;" => current_text.push(' '),
                        _ => current_text.push_str(&entity),
                    }
                    pos = end + 1;
                    continue;
                }
            }
            current_text.push(c);
            pos += 1;
        } else {
            pos += 1;
        }
    }
    flush(&mut current_text, &current_heading, &mut blocks);
    blocks
}
