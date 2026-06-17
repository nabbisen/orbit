//! Streaming content hashing (RFC-004 §9.2: sha256, streamed, never
//! loading whole files into memory — NFR-023).

use orbok_core::OrbokResult;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// SHA-256 of a file's content, streamed in 64 KiB blocks, returned as
/// lowercase hex.
pub fn sha256_file(path: &Path) -> OrbokResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

/// SHA-256 of an in-memory byte slice (chunk hashes, tests).
pub fn sha256_bytes(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn hex(digest: &[u8]) -> String {
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}
