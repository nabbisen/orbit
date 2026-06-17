//! Synthetic benchmark corpus generation (RFC-016 §8).
//!
//! Generates realistic-looking Markdown files covering a range of
//! topics — authentication, storage, search, code, and mixed-language
//! content — so that recall benchmarks are meaningful.

use std::fs;
use std::path::Path;

/// Synthetic document templates.
const TEMPLATES: &[(&str, &str)] = &[
    ("auth", "# Authentication\n\n## Token Lifecycle\n\nRefresh tokens expire after 24 hours.\nError code ERR-4042 occurs when a token is missing.\n\n## OAuth Client Rotation\n\nThe `client_secret` must be rotated every 90 days.\n"),
    ("storage", "# Storage Policy\n\n## Index Types\n\nOrbok stores derived indexes, not full copies.\nThe `orbok-catalog.sqlite3` file holds persistent catalog data.\n\n## Cleanup\n\nSafe cleanup removes expired caches without touching source files.\n"),
    ("search", "# Search Architecture\n\n## Hybrid Retrieval\n\nKeyword search uses FTS5 with unicode61 tokenization.\nVector search uses exact cosine-similarity scan.\nReciprocal Rank Fusion (k=60) combines both result sets.\n"),
    ("api", "# API Reference\n\n## Endpoints\n\n### POST /api/search/query\n\nRuns a hybrid search query. Parameters: `query`, `mode`, `limit`.\n\n### GET /api/sources\n\nLists all registered sources.\n"),
    ("security", "# Security Model\n\n## File Access Boundary\n\nThe backend enforces a source allowlist.\nPath traversal and symlink escape are rejected.\n\n## Log Hygiene\n\nDocument contents are never written to logs.\n"),
    ("japanese", "# 認証とストレージ\n\n## トークンのローテーション\n\nOAuthクライアントシークレットは定期的に更新する必要があります。\nエラーコード ERR-4042 はトークン欠落を示します。\n\n## インデックス管理\n\norbok はソースファイルのコピーを保存しません。\n"),
    ("code", "# Implementation Notes\n\n## Chunker\n\nThe `chunk()` function splits extraction output into `ChunkSpec` values.\nParent chunk (ordinal 0) spans the whole document.\nChild chunks map to headings or paragraphs.\n\n```rust\npub fn chunk(output: &ExtractOutput, name: &str) -> Vec<ChunkSpec> { ... }\n```\n"),
    ("models", "# Local AI Models\n\n## Embedding Model\n\nThe embedding model converts text to dense vectors (768 dimensions).\nLocal inference runs via candle or ONNX Runtime.\n\n## Reranker\n\nThe cross-encoder reranker scores query-passage pairs.\nIt is optional; keyword results are returned if unavailable.\n"),
];

/// Generate `n` synthetic Markdown files under `dir`.
pub fn generate(dir: &Path, n: usize) -> std::io::Result<()> {
    for i in 0..n {
        let (name, base) = TEMPLATES[i % TEMPLATES.len()];
        let content = format!("{base}\n<!-- doc-{i} -->\n");
        fs::write(dir.join(format!("{name}-{i:04}.md")), &content)?;
    }
    Ok(())
}

/// Total bytes of all Markdown files under `dir`.
pub fn total_bytes(dir: &Path) -> u64 {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .filter_map(|e| fs::metadata(e.path()).ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}
