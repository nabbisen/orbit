//! Build script: embed version and build metadata (RFC-017 §14).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=ORBOK_BUILD_DATE={}", chrono_or_static());
}

fn chrono_or_static() -> &'static str {
    // Use a static date; a real build would use chrono or `time`.
    "2026-06-07"
}
