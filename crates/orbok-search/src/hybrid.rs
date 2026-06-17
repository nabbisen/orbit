//! Hybrid search service (RFC-009): combines keyword and vector
//! retrieval through RRF fusion. Degrades gracefully when either source
//! is unavailable (RFC-009 §21).

use crate::fts5::Fts5KeywordEngine;
use crate::rrf::{FusedCandidate, rrf_fuse};
use crate::service::{MatchBadge, SearchResult};
use crate::snippet::{chunk_record_for, load_snippet};
use crate::vector::ExactVectorSearch;
use crate::KeywordSearchEngine;
use orbok_core::{ChunkId, FileId, OrbokResult};
use orbok_db::Catalog;
use orbok_models::{EmbeddingModel, l2_normalize};
use std::path::Path;

/// Search mode selector (RFC-009 §8, GUI design §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// Keyword + vector, RRF fused.
    #[default]
    Auto,
    /// Keyword-first; vector disabled.
    Exact,
    /// Vector-first; keyword disabled.
    Conceptual,
    /// Reduced candidate counts; no reranking.
    Fast,
}

/// Candidate limits per mode (RFC-009 §17).
struct Limits {
    keyword_k: u32,
    vector_k: u32,
    fusion_n: usize,
}

impl Limits {
    fn for_mode(mode: SearchMode) -> Self {
        match mode {
            SearchMode::Auto => Limits { keyword_k: 100, vector_k: 100, fusion_n: 50 },
            SearchMode::Exact => Limits { keyword_k: 100, vector_k: 0, fusion_n: 50 },
            SearchMode::Conceptual => Limits { keyword_k: 0, vector_k: 100, fusion_n: 50 },
            SearchMode::Fast => Limits { keyword_k: 50, vector_k: 50, fusion_n: 20 },
        }
    }
}

/// Hybrid search service. `embedding_model` is `None` when no model is
/// installed — the service falls back to keyword-only silently
/// (RFC-009 §21: "Search works without embedding model").
pub struct HybridSearchService<'a> {
    catalog: &'a Catalog,
    embedding_model: Option<(&'a dyn EmbeddingModel, String)>,
}

impl<'a> HybridSearchService<'a> {
    /// Keyword-only mode (no embedding model).
    pub fn keyword_only(catalog: &'a Catalog) -> Self {
        Self { catalog, embedding_model: None }
    }

    /// Hybrid mode with an embedding model (name+version determine
    /// which embeddings are eligible).
    pub fn with_model(
        catalog: &'a Catalog,
        model: &'a dyn EmbeddingModel,
        model_id: &str,
    ) -> Self {
        Self {
            catalog,
            embedding_model: Some((model, model_id.to_string())),
        }
    }

    pub fn is_hybrid(&self) -> bool {
        self.embedding_model.is_some()
    }

    /// Execute a search and return enriched results.
    pub fn search(
        &self,
        query: &str,
        mode: SearchMode,
        limit: u32,
    ) -> OrbokResult<Vec<SearchResult>> {
        let limits = Limits::for_mode(mode);

        // Keyword candidates.
        let kw_candidates = if limits.keyword_k > 0 {
            Fts5KeywordEngine::new(self.catalog).search(query, limits.keyword_k)?
        } else {
            Vec::new()
        };

        // Vector candidates.
        let vec_candidates = if limits.vector_k > 0 {
            if let Some((model, model_id)) = &self.embedding_model {
                let mut query_vec = model.embed_batch(&[query])?.remove(0);
                l2_normalize(&mut query_vec);
                ExactVectorSearch {
                    catalog: self.catalog,
                    model_id: model_id.clone(),
                    dimension: model.dimension(),
                }
                .search(&query_vec, limits.vector_k)?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Fuse.
        let fused = rrf_fuse(&kw_candidates, &vec_candidates, limits.fusion_n);

        // Enrich top results with snippets.
        let mut results = Vec::new();
        for candidate in fused.iter().take(limit as usize) {
            if let Some(result) = self.enrich(candidate)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    fn enrich(&self, candidate: &FusedCandidate) -> OrbokResult<Option<SearchResult>> {
        let Some((chunk, canonical_path)) =
            chunk_record_for(self.catalog, &candidate.chunk_id)?
        else {
            return Ok(None);
        };
        let snippet = load_snippet(&chunk, &canonical_path);
        let display_path = short_display_path(&canonical_path);
        let title = chunk.heading_path.clone().or_else(|| {
            Path::new(&canonical_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        });
        let mut badges = Vec::new();
        if candidate.keyword_rank.is_some() {
            badges.push(MatchBadge::Keyword);
        }
        if candidate.vector_rank.is_some() {
            badges.push(MatchBadge::Semantic);
        }
        Ok(Some(SearchResult {
            chunk_id: candidate.chunk_id.clone(),
            file_id: candidate.file_id.clone(),
            canonical_path,
            display_path,
            title,
            heading_path: chunk.heading_path,
            snippet,
            keyword_rank: candidate.keyword_rank.unwrap_or(0),
            keyword_score: 0.0,
            badges,
        }))
    }
}

fn short_display_path(path: &str) -> String {
    let p = Path::new(path);
    let parts: Vec<_> = p.components().collect();
    if parts.len() <= 2 { return path.to_string(); }
    let tail: std::path::PathBuf = parts[parts.len() - 2..].iter().collect();
    format!("…/{}", tail.display())
}
