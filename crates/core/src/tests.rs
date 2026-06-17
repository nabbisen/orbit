//! Tests for orbok-core. Test cases validate the design specifications
//! (RFC-001 acceptance criteria, RFC-002/004 status vocabulary), not
//! merely the written code.

use crate::data_class::{CleanupAction, CleanupPlan, DataClass, StorageCategory};
use crate::id::{FileId, SourceId};
use crate::status::{FileStatus, HiddenFilePolicy, SourceStatus, SymlinkPolicy};
use crate::timeutil::now_iso8601;

// RFC-001 §12: "Cleanup functions require a target lifecycle class" /
// "Ordinary cleanup cannot delete persistent source settings."
#[test]
fn safe_cleanup_actions_never_touch_persistent_catalog() {
    let safe = [
        CleanupAction::ClearExpiredSearchCache,
        CleanupAction::ClearSnippetCache,
        CleanupAction::ClearTemporaryExtraction,
        CleanupAction::RemoveReplacedStaleIndexes,
    ];
    for action in safe {
        let plan = CleanupPlan::for_action(action, 0);
        assert!(
            plan.assert_safe_for_ordinary_cleanup().is_ok(),
            "{action:?} must be safe"
        );
        assert!(!plan.affected_classes.contains(&DataClass::PersistentCatalog));
    }
}

// RFC-001 §8.3: reset catalog is destructive and requires confirmation.
#[test]
fn reset_catalog_is_flagged_destructive() {
    let plan = CleanupPlan::for_action(CleanupAction::ResetCatalog, 0);
    assert!(plan.requires_confirmation);
    assert!(plan.assert_safe_for_ordinary_cleanup().is_err());
}

// RFC-001 §7.2: rebuildable index deletion marks required reindexing.
#[test]
fn index_deletion_requires_rebuild_and_confirmation() {
    for action in [
        CleanupAction::DeleteKeywordIndex,
        CleanupAction::DeleteVectorIndex,
        CleanupAction::RemoveTemporarySourceIndexes,
    ] {
        let plan = CleanupPlan::for_action(action, 1024);
        assert!(plan.requires_rebuild, "{action:?}");
        assert!(plan.requires_confirmation, "{action:?}");
        assert_eq!(plan.affected_classes, vec![DataClass::RebuildableIndex]);
    }
}

// RFC-001 §10: storage accounting reportable by lifecycle category; every
// category maps to exactly one lifecycle class.
#[test]
fn storage_categories_cover_rfc_001_list_and_map_to_classes() {
    let names: Vec<&str> = StorageCategory::ALL.iter().map(|c| c.as_str()).collect();
    for required in [
        "persistent_catalog",
        "keyword_index",
        "vector_index",
        "snippet_cache",
        "search_cache",
        "temporary_extraction",
        "model_files",
        "logs",
    ] {
        assert!(names.contains(&required), "missing category {required}");
    }
    assert_eq!(
        StorageCategory::PersistentCatalog.data_class(),
        DataClass::PersistentCatalog
    );
    assert_eq!(
        StorageCategory::VectorIndex.data_class(),
        DataClass::RebuildableIndex
    );
    assert_eq!(
        StorageCategory::SnippetCache.data_class(),
        DataClass::EphemeralCache
    );
}

// RFC-004 §7: all eight file statuses representable; round-trip stable.
#[test]
fn file_status_round_trips() {
    for s in [
        FileStatus::Discovered,
        FileStatus::Indexed,
        FileStatus::Stale,
        FileStatus::Missing,
        FileStatus::Deleted,
        FileStatus::PermissionDenied,
        FileStatus::Unsupported,
        FileStatus::Failed,
    ] {
        assert_eq!(FileStatus::parse(s.as_str()).unwrap(), s);
    }
    assert!(FileStatus::parse("bogus").is_err());
}

// RFC-003 §6: defaults are the safe choices.
#[test]
fn safe_policy_defaults() {
    assert_eq!(HiddenFilePolicy::default(), HiddenFilePolicy::Exclude);
    assert_eq!(SymlinkPolicy::default(), SymlinkPolicy::Ignore);
}

#[test]
fn source_status_vocabulary_complete() {
    for s in ["active", "paused", "missing", "permission_denied", "removed"] {
        assert!(SourceStatus::parse(s).is_ok(), "{s}");
    }
}

// External design §9.2: prefixed, unique, time-ordered IDs.
#[test]
fn typed_ids_are_prefixed_and_unique() {
    let a = SourceId::generate();
    let b = SourceId::generate();
    assert!(a.as_str().starts_with("src_"));
    assert_ne!(a, b);
    let f = FileId::generate();
    assert!(f.as_str().starts_with("file_"));
}

// External design §9.3: UTC ISO-8601.
#[test]
fn timestamps_are_iso8601_utc() {
    let t = now_iso8601();
    assert!(t.ends_with('Z'), "expected UTC Z suffix: {t}");
    assert!(t.contains('T'));
}
