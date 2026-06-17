//! Sensitive-directory warnings (RFC-003 §7, external design §18.2).
//!
//! Warnings, not blocks: the user may proceed with "Add Anyway", but the
//! default recommendation is not to index credential-bearing folders.

use std::path::Path;

/// Directory names (final or intermediate components) that very likely
/// contain credentials or secrets.
const SENSITIVE_COMPONENTS: &[&str] = &[
    ".ssh",
    ".gnupg",
    ".aws",
    ".azure",
    ".kube",
    ".docker",
    ".password-store",
    ".mozilla",
    ".thunderbird",
];

/// Absolute prefixes that are system directories.
#[cfg(unix)]
const SYSTEM_PREFIXES: &[&str] = &["/etc", "/usr", "/bin", "/sbin", "/boot", "/proc", "/sys"];

#[cfg(not(unix))]
const SYSTEM_PREFIXES: &[&str] = &["C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)"];

/// Returns a warning reason when `path` looks like a sensitive location.
/// `None` means no warning is needed.
pub fn sensitive_warning(path: &Path) -> Option<&'static str> {
    let path_str = path.to_string_lossy();
    for prefix in SYSTEM_PREFIXES {
        if path_str.starts_with(prefix) {
            return Some("system_directory");
        }
    }
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if SENSITIVE_COMPONENTS.contains(&name.as_ref()) {
            return Some("credential_directory");
        }
        // `.config` only as the home config root, not arbitrary names.
        if name == ".config" {
            return Some("hidden_configuration_directory");
        }
    }
    None
}
