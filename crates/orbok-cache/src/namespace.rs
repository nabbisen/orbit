//! Cache namespaces (Appendix A §7).
//!
//! Namespace strings carry an explicit schema version suffix; payload
//! shape changes bump the version so `purge_stale_versions` can retire
//! old rows safely.

use orbok_core::DataClass;

/// The orbok cache namespaces. Embedding bundles are parameterized by
/// model and vector format so different models never collide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrbokCacheNamespace {
    /// Extracted, normalized segments per source file (RFC-005 output).
    ExtractSegments,
    /// Chunk bundles per source file (RFC-006 output).
    ChunkBundle,
    /// Embedding bundles per source file for one model+format (RFC-008).
    EmbeddingBundle {
        model_id: String,
        vector_format: String,
    },
    /// Rendered preview/snippet payloads (RFC-013 preview pane).
    PreviewCache,
}

impl OrbokCacheNamespace {
    /// The localcache namespace string (Appendix A §7 table).
    pub fn as_namespace(&self) -> String {
        match self {
            Self::ExtractSegments => "extract-segments:v1".to_string(),
            Self::ChunkBundle => "chunk-bundle:v1".to_string(),
            Self::EmbeddingBundle {
                model_id,
                vector_format,
            } => format!("embedding-bundle:{model_id}:{vector_format}:v1"),
            Self::PreviewCache => "preview-cache:v1".to_string(),
        }
    }

    /// localcache payload version for `purge_stale_versions`.
    pub fn payload_version(&self) -> u32 {
        1
    }

    /// Lifecycle class of the payloads (RFC-001 §5, Appendix A §6):
    /// derived pipeline payloads are rebuildable; previews are ephemeral.
    pub fn data_class(&self) -> DataClass {
        match self {
            Self::ExtractSegments | Self::ChunkBundle | Self::EmbeddingBundle { .. } => {
                DataClass::RebuildableIndex
            }
            Self::PreviewCache => DataClass::EphemeralCache,
        }
    }
}
