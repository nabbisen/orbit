//! Text normalization, version `norm-v1` (RFC-005 §9).
//!
//! norm-v1 is deliberately small and exactly specified, because its
//! output feeds content hashes and indexes — any change must come with
//! a version bump:
//! 1. strip a UTF-8 BOM at the start of the document;
//! 2. normalize CRLF and lone CR to LF;
//! 3. remove control characters except `\n` and `\t`;
//! 4. trim trailing whitespace on each line.
//!
//! Unicode NFC normalization is intentionally **not** part of norm-v1
//! (deferred to a future norm-v2 with RFC-014 language work); Japanese
//! text passes through byte-identical apart from the rules above.

/// Version constant recorded with every extraction.
pub use orbit_core::versions::NORMALIZATION_VERSION;

/// Apply norm-v1 to a whole document.
pub fn normalize_document(input: &str) -> String {
    let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let mut out = String::with_capacity(input.len());
    let mut line = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                flush_line(&mut out, &mut line);
            }
            '\n' => flush_line(&mut out, &mut line),
            '\t' => line.push('\t'),
            c if c.is_control() => {} // rule 3
            c => line.push(c),
        }
    }
    if !line.is_empty() {
        out.push_str(line.trim_end());
    }
    out
}

fn flush_line(out: &mut String, line: &mut String) {
    out.push_str(line.trim_end());
    out.push('\n');
    line.clear();
}
