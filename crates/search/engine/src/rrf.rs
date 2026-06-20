//! Reciprocal Rank Fusion (RFC-009 §7).
//!
//! Standard RRF formula: score(d) = Σ 1 / (k + rank_i(d))
//! Default k = 60 (validated in information-retrieval literature).

use orbok_core::{ChunkId, FileId};
use orbok_models::VectorCandidate;

use crate::KeywordCandidate;

/// One fused candidate carrying per-source ranks and the combined score.
#[derive(Debug, Clone)]
pub struct FusedCandidate {
    pub chunk_id: ChunkId,
    pub file_id: FileId,
    pub rrf_score: f64,
    pub keyword_rank: Option<u32>,
    pub vector_rank: Option<u32>,
}

/// RRF k constant (RFC-009 §7).
pub const RRF_K: f64 = 60.0;

/// Fuse keyword and vector candidates using Reciprocal Rank Fusion.
/// Returns candidates in descending RRF score order, deduplicated by
/// chunk_id (RFC-009 §9 deduplication).
pub fn rrf_fuse(
    keyword: &[KeywordCandidate],
    vector: &[VectorCandidate],
    limit: usize,
) -> Vec<FusedCandidate> {
    use std::collections::HashMap;
    let mut scores: HashMap<String, FusedCandidate> = HashMap::new();

    for kw in keyword {
        let key = kw.chunk_id.as_str().to_string();
        let entry = scores.entry(key).or_insert_with(|| FusedCandidate {
            chunk_id: kw.chunk_id.clone(),
            file_id: kw.file_id.clone(),
            rrf_score: 0.0,
            keyword_rank: None,
            vector_rank: None,
        });
        entry.rrf_score += 1.0 / (RRF_K + kw.rank as f64);
        entry.keyword_rank = Some(kw.rank);
    }

    for vc in vector {
        let key = vc.chunk_id.as_str().to_string();
        let entry = scores.entry(key).or_insert_with(|| FusedCandidate {
            chunk_id: vc.chunk_id.clone(),
            file_id: vc.file_id.clone(),
            rrf_score: 0.0,
            keyword_rank: None,
            vector_rank: None,
        });
        entry.rrf_score += 1.0 / (RRF_K + vc.rank as f64);
        entry.vector_rank = Some(vc.rank);
    }

    let mut fused: Vec<FusedCandidate> = scores.into_values().collect();
    fused.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused.truncate(limit);
    fused
}
