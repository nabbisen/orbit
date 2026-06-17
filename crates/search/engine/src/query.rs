//! Safe FTS5 MATCH expression building (RFC-015 §13: user input is
//! data, not query syntax).
//!
//! Each whitespace-separated term becomes a double-quoted phrase with
//! embedded quotes doubled, joined with implicit AND. FTS5 operators in
//! user input (`OR`, `NEAR`, `-`, `^`, column filters) are thereby
//! neutralized into literal phrases.

/// Build an FTS5 MATCH expression from a raw user query. Returns `None`
/// when the query contains no searchable terms.
pub fn build_match_expression(raw: &str) -> Option<String> {
    let mut phrases = Vec::new();
    for term in raw.split_whitespace() {
        let cleaned = term.replace('"', "\"\"");
        if cleaned.is_empty() {
            continue;
        }
        phrases.push(format!("\"{cleaned}\""));
    }
    if phrases.is_empty() {
        None
    } else {
        Some(phrases.join(" "))
    }
}
