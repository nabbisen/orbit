//! HTML text extractor (RFC-005 §5; RFC-044 §16.3 resource limits).
//!
//! Strips HTML tags with a simple state-machine parser and preserves
//! visible text content. Block-level elements produce paragraph
//! boundaries. `<h1>`–`<h6>` headings populate `heading_path`.
//!
//! Security: no JavaScript execution, no external resource loading,
//! no DOM construction. Pure text extraction only (RFC-015 §15).

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractContext, ExtractOutput, ExtractWarning, ExtractedSegment,
    LocationKind, LocationQuality, SegmentKind, read_error_category,
};
use orbok_core::{ErrorCategory, OrbokError, OrbokResult, versions::NORMALIZATION_VERSION};
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

    fn extract_with_context(
        &self,
        path: &ValidatedPath,
        context: &ExtractContext,
    ) -> OrbokResult<ExtractOutput> {
        let limits = &context.limits;
        let mut warnings = Vec::new();

        // RFC-044 §9.5: check file size before reading.
        let meta = std::fs::metadata(&path.canonical).map_err(|e| OrbokError::Extraction {
            category: read_error_category(&e),
            message: e.to_string(),
        })?;
        if meta.len() > limits.max_html_bytes {
            return Err(OrbokError::Extraction {
                category: ErrorCategory::FileTooLarge,
                message: format!(
                    "HTML file is {} bytes, limit is {}",
                    meta.len(),
                    limits.max_html_bytes
                ),
            });
        }

        let content =
            std::fs::read_to_string(&path.canonical).map_err(|e| OrbokError::Extraction {
                category: read_error_category(&e),
                message: e.to_string(),
            })?;

        let blocks = extract_blocks(&content);
        let mut segments = Vec::new();
        let mut total_chars = 0u64;
        let mut block_idx = 1u32;

        for block in &blocks {
            let norm = normalize_document(&block.text);
            if norm.trim().is_empty() {
                block_idx += 1;
                continue;
            }

            // RFC-044 §9.5: extracted char limit — stop and warn.
            let block_chars = norm.chars().count() as u64;
            if total_chars + block_chars > limits.max_extracted_chars {
                warnings.push(ExtractWarning::SizeLimitReached {
                    limit_name: "max_extracted_chars".into(),
                });
                break;
            }
            total_chars += block_chars;

            segments.push(ExtractedSegment {
                kind: block.kind,
                text: norm,
                line_start: block_idx,
                line_end: block_idx,
                location_kind: LocationKind::Blocks,
                heading_path: block.heading.clone(),
                location_quality: LocationQuality::Approximate,
            });
            block_idx += 1;

            // RFC-044 §9.5: segment count limit.
            if segments.len() >= limits.max_segments {
                warnings.push(ExtractWarning::SizeLimitReached {
                    limit_name: "max_segments".into(),
                });
                break;
            }
        }

        Ok(ExtractOutput {
            extractor_name: EXTRACTOR_NAME.to_string(),
            extractor_version: EXTRACTOR_VERSION.to_string(),
            normalization_version: NORMALIZATION_VERSION.to_string(),
            segments,
            char_count: total_chars,
            warnings,
        })
    }

    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        self.extract_with_context(path, &ExtractContext::default())
    }
}

struct Block {
    kind: SegmentKind,
    text: String,
    heading: Option<String>,
}

/// Extract text blocks from HTML using a simple state machine.
fn extract_blocks(html: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut current = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut current_kind = SegmentKind::Paragraph;
    let mut heading_stack: Vec<String> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut skip_depth = 0u32; // for script/style nesting

    let push_block = |blocks: &mut Vec<Block>, text: &str, kind, heading: Option<String>| {
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            blocks.push(Block {
                kind,
                text: trimmed,
                heading,
            });
        }
    };

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            tag_name.clear();
            continue;
        }
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let tag = tag_name.trim().to_ascii_lowercase();
                let (closing, base) = if let Some(b) = tag.strip_prefix('/') {
                    (true, b.trim().to_string())
                } else {
                    (
                        false,
                        tag.split_whitespace().next().unwrap_or("").to_string(),
                    )
                };

                match base.as_str() {
                    "script" | "style" => {
                        if closing {
                            skip_depth = skip_depth.saturating_sub(1);
                        } else {
                            skip_depth += 1;
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if !closing => {
                        push_block(&mut blocks, &current, current_kind, current_heading.clone());
                        current.clear();
                        current_kind = SegmentKind::Heading;
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if closing => {
                        let title = current.trim().to_string();
                        if !title.is_empty() {
                            heading_stack.retain(|h| h != &title);
                            heading_stack.push(title.clone());
                            current_heading = Some(heading_stack.join(" > "));
                            blocks.push(Block {
                                kind: SegmentKind::Heading,
                                text: title,
                                heading: current_heading.clone(),
                            });
                        }
                        current.clear();
                        current_kind = SegmentKind::Paragraph;
                    }
                    "p" | "div" | "li" | "td" | "th" | "article" | "section" | "blockquote" => {
                        if !closing {
                            push_block(
                                &mut blocks,
                                &current,
                                current_kind,
                                current_heading.clone(),
                            );
                            current.clear();
                            current_kind = SegmentKind::Paragraph;
                        }
                    }
                    "br" => {
                        current.push('\n');
                    }
                    _ => {}
                }
            } else {
                tag_name.push(ch);
            }
            continue;
        }
        // Text content.
        if skip_depth == 0 {
            current.push(ch);
        }
    }
    push_block(&mut blocks, &current, current_kind, current_heading);
    blocks
}
