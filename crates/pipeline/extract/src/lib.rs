//! # orbok-extract
//!
//! Text extraction (RFC-005): pluggable extractors turn boundary-
//! validated source files into normalized, location-tagged segments.
//! Extraction output is derived data — cacheable, rebuildable, never
//! authoritative.
//!
//! RFC-044 hardening adds: resource limits (`ExtractLimits`), structured
//! warnings (`ExtractWarning`), panic isolation (`extract_safely`),
//! explicit location semantics (`LocationKind`), and removal of the
//! `orbok-db` production dependency (chunker now produces
//! `ExtractedChunk`; the pipeline layer maps to `ChunkSpec`).

pub mod chunker;
pub mod normalize;
pub mod registry;
pub mod types;

pub mod docx;
pub mod html;
mod markdown;
pub mod pdf;
pub mod plugin;
mod text;

#[cfg(test)]
mod tests;

pub use chunker::chunk;
pub use plugin::{PluginManifest, PluginRegistry};
pub use registry::ExtractorRegistry;
pub use types::{
    DocumentExtractor, ExtractContext, ExtractLimits, ExtractOutput, ExtractWarning,
    ExtractedChunk, ExtractedSegment, LocationKind, LocationQuality, SegmentKind,
};
