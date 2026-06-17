//! # orbok-extract
//!
//! Text extraction (RFC-005): pluggable extractors turn boundary-
//! validated source files into normalized, line-located segments.
//! Extraction output is derived data — cacheable, rebuildable, never
//! authoritative.

pub mod chunker;
pub mod normalize;
pub mod registry;
pub mod types;

pub mod docx;
pub mod html;
mod markdown;
pub mod plugin;
pub mod pdf;
mod text;

#[cfg(test)]
mod tests;

pub use registry::ExtractorRegistry;
pub use chunker::chunk;
pub use plugin::{PluginManifest, PluginRegistry};
pub use types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
};
