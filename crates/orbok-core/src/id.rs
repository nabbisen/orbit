//! Typed application-level identifiers.
//!
//! External design §9.2: application-level IDs are UUIDv7 strings with a
//! readable prefix; external interfaces expose opaque strings rather than
//! SQLite row ids. The newtype-per-entity pattern prevents accidentally
//! passing a `FileId` where a `SourceId` is expected.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! typed_id {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Generate a fresh identifier (UUIDv7, time-ordered).
            pub fn generate() -> Self {
                Self(format!("{}_{}", $prefix, uuid::Uuid::now_v7()))
            }

            /// Wrap an existing identifier string (e.g. read from the catalog).
            pub fn from_string(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            /// Borrow the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }
    };
}

typed_id!(
    /// Identifier of a registered source (RFC-003).
    SourceId,
    "src"
);
typed_id!(
    /// Identifier of a cataloged file (RFC-004).
    FileId,
    "file"
);
typed_id!(
    /// Identifier of an extraction record (RFC-005).
    ExtractionId,
    "ext"
);
typed_id!(
    /// Identifier of a chunk (RFC-006).
    ChunkId,
    "chunk"
);
typed_id!(
    /// Identifier of an index job (RFC-002 §7.9).
    JobId,
    "job"
);
typed_id!(
    /// Identifier of a registered local model (RFC-012).
    ModelId,
    "model"
);
typed_id!(
    /// Identifier of a search query record (RFC-002 §7.10).
    QueryId,
    "query"
);
typed_id!(
    /// Identifier of an application event (RFC-002 §7.13).
    EventId,
    "evt"
);
