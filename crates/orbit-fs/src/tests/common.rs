//! Shared fixtures for orbit-fs tests.

use orbit_core::{
    HiddenFilePolicy, IndexMode, PersistenceMode, SourceId, SourceType, SymlinkPolicy,
};
use orbit_db::Catalog;
use orbit_db::repo::{NewSource, SourceRecord, SourceRepository};
use std::path::Path;

/// Register a directory source over `root` with the safe defaults and
/// return its record.
pub fn register_dir_source(catalog: &Catalog, root: &Path) -> SourceRecord {
    register_dir_source_with(catalog, root, HiddenFilePolicy::Exclude, SymlinkPolicy::Ignore)
}

pub fn register_dir_source_with(
    catalog: &Catalog,
    root: &Path,
    hidden: HiddenFilePolicy,
    symlink: SymlinkPolicy,
) -> SourceRecord {
    let canonical = std::fs::canonicalize(root).unwrap();
    SourceRepository::new(catalog)
        .insert(NewSource {
            source_type: SourceType::Directory,
            persistence_mode: PersistenceMode::Persistent,
            display_name: None,
            original_path: root.to_string_lossy().into_owned(),
            canonical_path: canonical.to_string_lossy().into_owned(),
            index_mode: IndexMode::Balanced,
            include_patterns: vec![],
            exclude_patterns: vec![],
            hidden_file_policy: hidden,
            symlink_policy: symlink,
            max_file_size_bytes: None,
        })
        .unwrap()
}

/// Scan helper with default request flags.
pub fn scan(catalog: &Catalog, source_id: &SourceId) -> crate::ScanSummary {
    let scanner = crate::Scanner::new(catalog);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    scanner
        .scan(
            &crate::ScanRequest {
                source_id: source_id.clone(),
                force_hash: false,
                enqueue_index_jobs: true,
            },
            &cancel,
        )
        .unwrap()
}
