//! # orbok-fs
//!
//! The safe file-access layer:
//! - [`path_guard`] — backend-enforced source membership and policy
//!   validation (RFC-003 §8): no file is read unless it passes;
//! - [`policy`] — compiled source policies (include/exclude, hidden,
//!   symlink, size, supported types);
//! - [`sensitive`] — sensitive-directory warnings (RFC-003 §7);
//! - [`scanner`] — file discovery and change detection (RFC-004).
//!
//! The GUI never calls into this crate; it goes through `orbok-core`
//! service interfaces (RFC-027 boundary rule).

pub mod hashing;
pub mod path_guard;
pub mod policy;
pub mod scanner;
pub mod sensitive;

#[cfg(test)]
mod tests;

pub use path_guard::{GuardedSource, PathGuard, ValidatedPath};
pub use policy::{CompiledPolicy, FileTypeClass};
pub use scanner::{ScanOutcomeKind, ScanRequest, ScanSummary, Scanner};
pub use sensitive::sensitive_warning;
