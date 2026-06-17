//! Pipeline version constants.
//!
//! Versioned stages (RFC-005 §9, RFC-006 §13, RFC-007 §9): bumping any
//! of these marks dependent records stale and eligible for reindexing.

/// Text normalization stage version (RFC-005 §9).
pub const NORMALIZATION_VERSION: &str = "norm-v1";

/// Chunker version (RFC-006 §13). Reserved; chunking lands in M5.
pub const CHUNKER_VERSION: &str = "chunker-v1";

/// Keyword index text builder version (RFC-007 §9).
pub const KEYWORD_TEXT_BUILDER_VERSION: &str = "kw-text-v1";

/// Embedding text builder version (RFC-008 §7). Reserved for M7.
pub const EMBEDDING_TEXT_BUILDER_VERSION: &str = "embed-text-v1";
