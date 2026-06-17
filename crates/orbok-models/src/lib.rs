//! # orbok-models
//!
//! Local AI model vocabulary (RFC-012). Milestone M1–M6 only needs the
//! shared types and the "what is available" summary the UI shows; the
//! install/locate/validate workflow lands in M12.
//!
//! Privacy rule carried from the requirements: model *download* is the
//! only network operation orbok may ever perform, it is explicit, and
//! it never involves document contents.

use serde::{Deserialize, Serialize};

/// Model roles (catalog `models.role`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Embedding,
    Reranker,
}

impl ModelRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelRole::Embedding => "embedding",
            ModelRole::Reranker => "reranker",
        }
    }
}

/// Model availability (catalog `models.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    Available,
    Missing,
    Invalid,
    Installing,
    Disabled,
}

impl ModelStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelStatus::Available => "available",
            ModelStatus::Missing => "missing",
            ModelStatus::Invalid => "invalid",
            ModelStatus::Installing => "installing",
            ModelStatus::Disabled => "disabled",
        }
    }
}

/// Search capability derived from model availability. Keyword search
/// never depends on models (RFC-007: works with zero models installed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCapability {
    /// Keyword only: no embedding model available.
    KeywordOnly,
    /// Keyword + semantic: embedding model available.
    Hybrid,
    /// Keyword + semantic + rerank refinement.
    HybridWithRerank,
}

/// Derive the capability shown in the UI from model statuses.
pub fn search_capability(
    embedding: Option<ModelStatus>,
    reranker: Option<ModelStatus>,
) -> SearchCapability {
    match (embedding, reranker) {
        (Some(ModelStatus::Available), Some(ModelStatus::Available)) => {
            SearchCapability::HybridWithRerank
        }
        (Some(ModelStatus::Available), _) => SearchCapability::Hybrid,
        _ => SearchCapability::KeywordOnly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC-007/RFC-010: search degrades gracefully without models.
    #[test]
    fn capability_degrades_gracefully() {
        assert_eq!(search_capability(None, None), SearchCapability::KeywordOnly);
        assert_eq!(
            search_capability(Some(ModelStatus::Missing), None),
            SearchCapability::KeywordOnly
        );
        assert_eq!(
            search_capability(Some(ModelStatus::Available), None),
            SearchCapability::Hybrid
        );
        assert_eq!(
            search_capability(Some(ModelStatus::Available), Some(ModelStatus::Missing)),
            SearchCapability::Hybrid
        );
        assert_eq!(
            search_capability(Some(ModelStatus::Available), Some(ModelStatus::Available)),
            SearchCapability::HybridWithRerank
        );
    }
}

/// A vector search candidate (RFC-008 §13).
#[derive(Debug, Clone)]
pub struct VectorCandidate {
    pub chunk_id: orbok_core::ChunkId,
    pub file_id: orbok_core::FileId,
    pub rank: u32,
    pub score: f32,
}

/// Local embedding model abstraction (RFC-008 §6).
///
/// Implementations must not transmit text externally (NFR-001).
pub trait EmbeddingModel: Send + Sync {
    /// Stable name stored in `models.model_name`.
    fn name(&self) -> &str;
    /// Version string stored in `models.model_version`.
    fn version(&self) -> &str;
    /// Output dimension — must match stored embeddings (RFC-008 §11).
    fn dimension(&self) -> u32;
    /// Embed a batch of normalized texts. Returns one vector per input,
    /// each L2-normalized.
    fn embed_batch(&self, texts: &[&str]) -> orbok_core::OrbokResult<Vec<Vec<f32>>>;
}

/// Compute cosine similarity between two L2-normalized vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// L2-normalize a vector in-place. No-op for the zero vector.
pub fn l2_normalize(v: &mut Vec<f32>) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Serialize a vector to little-endian bytes for BLOB storage (RFC-008
/// §12.1 "sqlite_blob with FP32").
pub fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// Deserialize from BLOB bytes; returns `None` on length mismatch.
pub fn blob_to_vec(blob: &[u8], expected_dim: u32) -> Option<Vec<f32>> {
    let dim = expected_dim as usize;
    if blob.len() != dim * 4 {
        return None;
    }
    Some(
        blob.chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
    )
}

// ── Mock model ──────────────────────────────────────────────────────

/// Deterministic 8-dimensional mock embedding model.
///
/// Uses the SHA-256 of the input text as a pseudo-random source for 8
/// f32 components, then L2-normalizes the result.  **Never use for
/// semantic search** — the outputs are semantically meaningless.
/// Suitable for pipeline correctness tests (RFC-008 §24 tests 1–10).
pub struct MockEmbeddingModel;

impl EmbeddingModel for MockEmbeddingModel {
    fn name(&self) -> &str {
        "mock"
    }
    fn version(&self) -> &str {
        "v1"
    }
    fn dimension(&self) -> u32 {
        8
    }
    fn embed_batch(&self, texts: &[&str]) -> orbok_core::OrbokResult<Vec<Vec<f32>>> {
        use sha2::{Digest, Sha256};
        texts
            .iter()
            .map(|text| {
                let digest = Sha256::digest(text.as_bytes());
                let mut v: Vec<f32> = digest[..8]
                    .iter()
                    .map(|&b| b as f32 / 255.0)
                    .collect();
                l2_normalize(&mut v);
                Ok(v)
            })
            .collect()
    }
}

#[cfg(test)]
mod embedding_tests {
    use super::*;

    // RFC-008 §24 test 2: embedding generation succeeds for sample chunks.
    #[test]
    fn mock_embed_batch() {
        let model = MockEmbeddingModel;
        let vecs = model.embed_batch(&["hello world", "foo bar"]).unwrap();
        assert_eq!(vecs.len(), 2);
        for v in &vecs {
            assert_eq!(v.len(), model.dimension() as usize);
        }
    }

    // RFC-008 §24 test 3: dimension mismatch can be detected by caller.
    #[test]
    fn blob_roundtrip_and_dim_mismatch() {
        let v = vec![0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let blob = vec_to_blob(&v);
        assert_eq!(blob.len(), 32);
        let back = blob_to_vec(&blob, 8).unwrap();
        for (a, b) in v.iter().zip(&back) {
            assert!((a - b).abs() < 1e-6);
        }
        assert!(blob_to_vec(&blob, 16).is_none(), "dim mismatch must return None");
    }

    // L2 normalization: unit-length vectors.
    #[test]
    fn normalize_produces_unit_vector() {
        let mut v = vec![3.0_f32, 4.0];
        l2_normalize(&mut v);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    // RFC-008 §24 test 9: cosine sim of identical vectors = 1.0.
    #[test]
    fn cosine_sim_identical_vectors() {
        let mut v = vec![1.0_f32, 2.0, 3.0];
        l2_normalize(&mut v);
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }
}
