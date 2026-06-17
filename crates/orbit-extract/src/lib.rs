//! # orbit-extract
//!
//! Text extraction (RFC-005): pluggable extractors turn boundary-
//! validated source files into normalized, line-located segments.
//! Extraction output is derived data — cacheable, rebuildable, never
//! authoritative.

pub mod normalize;
pub mod registry;
pub mod types;

mod markdown;
mod text;

#[cfg(test)]
mod tests;

pub use registry::ExtractorRegistry;
pub use types::{
    DocumentExtractor, ExtractOutput, ExtractedSegment, LocationQuality, SegmentKind,
};
