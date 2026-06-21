//! Adapter: `ExtractedChunk` → `orbok_db::repo::ChunkSpec` (RFC-044 §14).
//!
//! This conversion lives in `orbok-workers` (not in `orbok-extract`)
//! so that the extraction crate has no dependency on `orbok-db`.
//! The `location_kind` field is carried through the pipeline but is not
//! yet persisted to a dedicated DB column (that comes with later RFC
//! work on result trust and snippet loading).

use orbok_db::repo::ChunkSpec;
use orbok_extract::ExtractedChunk;

/// Convert one `ExtractedChunk` to a `ChunkSpec`.
pub fn to_chunk_spec(c: ExtractedChunk) -> ChunkSpec {
    ChunkSpec {
        chunk_kind: c.chunk_kind,
        chunk_ordinal: c.chunk_ordinal,
        heading_path: c.heading_path,
        title: c.title,
        normalized_text: c.normalized_text,
        line_start: c.line_start,
        line_end: c.line_end,
        byte_start: c.byte_start,
        byte_end: c.byte_end,
        location_quality: c.location_quality,
        parent_idx: c.parent_idx,
    }
}

/// Convert a `Vec<ExtractedChunk>` to a `Vec<ChunkSpec>`.
pub fn to_chunk_specs(chunks: Vec<ExtractedChunk>) -> Vec<ChunkSpec> {
    chunks.into_iter().map(to_chunk_spec).collect()
}
