//! DOCX text extractor (Microsoft Word 2007+).
//!
//! DOCX files are ZIP archives containing XML. This extractor reads
//! `word/document.xml` and strips XML tags to recover paragraph text.
//! Location quality is `Approximate` — DOCX does not provide byte
//! offsets; only paragraph order is preserved.
//!
//! Security: the file is opened with `zip::ZipArchive` which bounds
//! reads to the archive contents. No external entity expansion.

use crate::normalize::normalize_document;
use crate::types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
};
use orbok_core::{ErrorCategory, OrbokError, OrbokResult, versions::NORMALIZATION_VERSION};
use orbok_fs::ValidatedPath;
use std::io::Read;

const EXTRACTOR_NAME: &str = "docx";
const EXTRACTOR_VERSION: &str = "v1";

pub struct DocxExtractor;

impl DocumentExtractor for DocxExtractor {
    fn name(&self) -> &'static str { EXTRACTOR_NAME }
    fn version(&self) -> &'static str { EXTRACTOR_VERSION }
    fn supported_extensions(&self) -> &'static [&'static str] { &["docx"] }

    fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        let file = std::fs::File::open(&path.canonical)?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| OrbokError::Extraction {
            category: ErrorCategory::ParserError,
            message: format!("docx zip: {e}"),
        })?;

        let xml = match zip.by_name("word/document.xml") {
            Ok(mut entry) => {
                let mut s = String::new();
                entry.read_to_string(&mut s).map_err(|e| OrbokError::Extraction {
                    category: ErrorCategory::ParserError,
                    message: format!("docx xml read: {e}"),
                })?;
                s
            }
            Err(_) => return Err(OrbokError::Extraction {
                category: ErrorCategory::UnsupportedFormat,
                message: "no word/document.xml in archive".into(),
            }),
        };

        // Extract text runs from w:p paragraphs.
        let paragraphs = extract_paragraphs(&xml);
        let mut segments = Vec::new();
        let mut total_chars = 0u64;

        for (para_idx, para_text) in paragraphs.iter().enumerate() {
            let norm = normalize_document(para_text);
            if norm.trim().is_empty() { continue; }
            total_chars += norm.len() as u64;
            segments.push(ExtractedSegment {
                kind: SegmentKind::Paragraph,
                text: norm,
                line_start: (para_idx + 1) as u32,
                line_end: (para_idx + 1) as u32,
                heading_path: None,
                location_quality: LocationQuality::Approximate,
            });
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

/// Extract paragraph text from DOCX word/document.xml by collecting
/// all `w:t` text runs within each `w:p` paragraph element.
fn extract_paragraphs(xml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current_para = String::new();
    let mut in_para = false;
    let mut pos = 0;
    let bytes = xml.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            // Find end of tag
            let end = bytes[pos..].iter().position(|&b| b == b'>').map(|p| pos + p + 1).unwrap_or(bytes.len());
            let tag = &xml[pos..end];
            if tag.starts_with("<w:p ") || tag == "<w:p>" {
                in_para = true;
                current_para.clear();
            } else if tag.starts_with("</w:p>") {
                if in_para && !current_para.trim().is_empty() {
                    paragraphs.push(current_para.trim().to_string());
                }
                in_para = false;
                current_para.clear();
            } else if in_para && (tag.starts_with("<w:t") || tag.starts_with("<w:t>")) {
                // Collect text until </w:t>
                let text_start = end;
                let text_end = xml[text_start..].find("</w:t>").map(|p| text_start + p).unwrap_or(text_start);
                let text = &xml[text_start..text_end];
                // Skip any nested tags in text content
                let clean: String = {
                    let mut t = String::new();
                    let mut in_tag = false;
                    for c in text.chars() {
                        match c {
                            '<' => in_tag = true,
                            '>' => in_tag = false,
                            _ if !in_tag => t.push(c),
                            _ => {}
                        }
                    }
                    t
                };
                current_para.push_str(&clean);
                pos = text_end;
                continue;
            }
            pos = end;
        } else {
            pos += 1;
        }
    }
    paragraphs
}
