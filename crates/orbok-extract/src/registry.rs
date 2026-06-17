//! Extractor registry (RFC-005 §6: selection by file type, typed
//! unsupported results).

use crate::docx::DocxExtractor;
use crate::html::HtmlExtractor;
use crate::markdown::MarkdownExtractor;
use crate::pdf::PdfExtractor;
use crate::text::PlainTextExtractor;
use crate::types::{DocumentExtractor, ExtractOutput};
use orbok_core::{ErrorCategory, OrbokError, OrbokResult};
use orbok_fs::ValidatedPath;

/// Registry of the available extractors. Markdown takes precedence over
/// plain text for `.md`; everything claims by extension.
pub struct ExtractorRegistry {
    extractors: Vec<Box<dyn DocumentExtractor>>,
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        Self {
            extractors: vec![Box::new(MarkdownExtractor), Box::new(DocxExtractor), Box::new(HtmlExtractor), Box::new(PlainTextExtractor), Box::new(PdfExtractor)],
        }
    }
}

impl ExtractorRegistry {
    /// The extractor claiming `extension`, if any.
    pub fn select(&self, extension: &str) -> Option<&dyn DocumentExtractor> {
        let ext = extension.to_ascii_lowercase();
        self.extractors
            .iter()
            .find(|e| e.supported_extensions().contains(&ext.as_str()))
            .map(|e| e.as_ref())
    }

    /// Extract a validated file. Unknown types are a typed
    /// `UnsupportedType` failure that workers record on the extraction
    /// record (RFC-005 §13) — never a panic, never a silent skip.
    pub fn extract(&self, path: &ValidatedPath) -> OrbokResult<ExtractOutput> {
        let extension = path
            .canonical
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        match self.select(extension) {
            Some(extractor) => {
                tracing::debug!(extractor = extractor.name(), "extracting");
                extractor.extract(path)
            }
            None => Err(OrbokError::Extraction {
                category: ErrorCategory::UnsupportedType,
                message: format!("no extractor for extension '{extension}'"),
            }),
        }
    }
}
