//! Local model file readiness check (RFC-043 §7).
//!
//! orbok checks local model files before starting any download. This
//! avoids re-downloading files that are already present and valid,
//! and prevents treating partially downloaded files as usable.
//!
//! The readiness check is intentionally lightweight (P0): existence,
//! regular-file check, and non-empty. Checksum validation (P1) is
//! layered on top when a manifest is available.

use std::path::Path;

// ── Per-file status ───────────────────────────────────────────────────

/// Status of one required model file (RFC-043 §7.2).
///
/// User-facing copy must not expose these names directly:
/// - `Ready`       → "Ready"
/// - `Missing`     → "Needed"
/// - `Partial`     → "Needs to finish"
/// - `Invalid`     → "Needs to be replaced"
/// - `CannotCheck` → "Could not check"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalFileStatus {
    /// File exists, is non-empty, and passes available validation.
    Ready,
    /// File does not exist.
    Missing,
    /// A `.part` temporary file exists but the final file is absent or empty.
    Partial,
    /// File exists but failed validation (empty, wrong type, corrupted).
    Invalid,
    /// File access failed unexpectedly (permissions etc.).
    CannotCheck,
}

impl LocalFileStatus {
    /// User-facing copy (RFC-043 §7.2, avoiding technical terms).
    pub fn user_label(&self) -> &'static str {
        match self {
            LocalFileStatus::Ready => "Ready",
            LocalFileStatus::Missing => "Needed",
            LocalFileStatus::Partial => "Needs to finish",
            LocalFileStatus::Invalid => "Needs to be replaced",
            LocalFileStatus::CannotCheck => "Could not check",
        }
    }

    /// Whether this file needs any download or repair work.
    pub fn needs_work(&self) -> bool {
        !matches!(self, LocalFileStatus::Ready)
    }
}

// ── Per-file readiness entry ──────────────────────────────────────────

/// Readiness information for one required model file (RFC-043 §10.2).
#[derive(Debug, Clone)]
pub struct FileReadiness {
    /// Logical name (e.g. "tokenizer", "model").
    pub logical_name: &'static str,
    /// Relative path within the model directory.
    pub relative_path: &'static str,
    /// Current status of the local file.
    pub status: LocalFileStatus,
}

// ── Model-level readiness ─────────────────────────────────────────────

/// Overall model readiness (RFC-043 §7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelReadiness {
    /// Every required file is present and valid.
    Ready,
    /// At least one file is missing.
    NeedsDownload,
    /// At least one file is partial or invalid (but none missing).
    NeedsRepair,
    /// File access failed unexpectedly.
    CannotCheck,
}

// ── Readiness report ──────────────────────────────────────────────────

/// Full readiness report for a model directory (RFC-043 §7.2–7.3).
#[derive(Debug, Clone)]
pub struct ModelReadinessReport {
    pub overall: ModelReadiness,
    pub files: Vec<FileReadiness>,
}

impl ModelReadinessReport {
    /// Files that require download or repair.
    pub fn files_needing_work(&self) -> Vec<&FileReadiness> {
        self.files
            .iter()
            .filter(|f| f.status.needs_work())
            .collect()
    }

    /// How many files are already ready.
    pub fn ready_count(&self) -> usize {
        self.files
            .iter()
            .filter(|f| f.status == LocalFileStatus::Ready)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.files.len()
    }
}

// ── Required files ────────────────────────────────────────────────────

/// Required files for the current default model (RFC-043 §6).
const REQUIRED_FILES: &[(&str, &str)] = &[
    ("tokenizer", "tokenizer.json"),
    ("model", "onnx/model.onnx"),
];

// ── Readiness check ───────────────────────────────────────────────────

/// Check local model files and return a readiness report (RFC-043 §7).
///
/// Called: at startup, before the wizard, before download, after
/// download, after the user chooses an existing folder, and on retry.
///
/// This is a pure filesystem check — no network access.
pub fn check_model_readiness(model_dir: &Path) -> ModelReadinessReport {
    let mut files = Vec::new();

    for (logical_name, relative_path) in REQUIRED_FILES {
        let full_path = model_dir.join(relative_path);
        let part_path = model_dir.join(format!("{relative_path}.part"));
        let status = check_single_file(&full_path, &part_path);
        files.push(FileReadiness {
            logical_name,
            relative_path,
            status,
        });
    }

    let overall = derive_overall_readiness(&files);
    ModelReadinessReport { overall, files }
}

fn check_single_file(path: &Path, part_path: &Path) -> LocalFileStatus {
    // Check the final path first.
    match std::fs::metadata(path) {
        Ok(meta) => {
            if !meta.is_file() {
                return LocalFileStatus::Invalid;
            }
            if meta.len() == 0 {
                return LocalFileStatus::Invalid;
            }
            // P0 validation passes: file exists, is a regular file, is non-empty.
            LocalFileStatus::Ready
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Final file missing — check for a .part file.
            if part_path.exists() {
                LocalFileStatus::Partial
            } else {
                LocalFileStatus::Missing
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => LocalFileStatus::CannotCheck,
        Err(_) => LocalFileStatus::CannotCheck,
    }
}

fn derive_overall_readiness(files: &[FileReadiness]) -> ModelReadiness {
    if files
        .iter()
        .any(|f| f.status == LocalFileStatus::CannotCheck)
    {
        return ModelReadiness::CannotCheck;
    }
    if files.iter().all(|f| f.status == LocalFileStatus::Ready) {
        return ModelReadiness::Ready;
    }
    if files.iter().any(|f| f.status == LocalFileStatus::Missing) {
        return ModelReadiness::NeedsDownload;
    }
    // Partial or Invalid but no Missing.
    ModelReadiness::NeedsRepair
}
