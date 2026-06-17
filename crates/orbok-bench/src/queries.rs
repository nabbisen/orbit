//! Labeled benchmark queries (RFC-016 §9).

/// One labeled query with expected relevant document name patterns.
pub struct LabeledQuery {
    pub query: &'static str,
    pub relevant_patterns: &'static [&'static str],
}

/// Static labeled query set for recall evaluation.
pub const LABELED_QUERIES: &[LabeledQuery] = &[
    LabeledQuery {
        query: "refresh token expiry",
        relevant_patterns: &["auth"],
    },
    LabeledQuery {
        query: "ERR-4042",
        relevant_patterns: &["auth", "japanese"],
    },
    LabeledQuery {
        query: "client_secret rotation",
        relevant_patterns: &["auth"],
    },
    LabeledQuery {
        query: "source allowlist path traversal",
        relevant_patterns: &["security"],
    },
    LabeledQuery {
        query: "FTS5 keyword search",
        relevant_patterns: &["search"],
    },
    LabeledQuery {
        query: "embedding model cosine similarity",
        relevant_patterns: &["models", "search"],
    },
    LabeledQuery {
        query: "ChunkSpec ordinal",
        relevant_patterns: &["code"],
    },
    LabeledQuery {
        query: "orbok-catalog.sqlite3",
        relevant_patterns: &["storage"],
    },
];
