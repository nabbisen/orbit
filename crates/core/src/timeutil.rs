//! UTC timestamp helpers (external design §9.3: ISO-8601 UTC strings).

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Current UTC time as an RFC 3339 / ISO-8601 string, e.g.
/// `2026-06-06T12:34:56.789Z`.
pub fn now_iso8601() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC 3339 formatting of the current UTC time cannot fail")
}

/// Convert a [`std::time::SystemTime`] (e.g. file mtime) to RFC 3339.
pub fn system_time_iso8601(t: std::time::SystemTime) -> String {
    OffsetDateTime::from(t)
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}
