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
