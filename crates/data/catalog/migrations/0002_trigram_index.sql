-- Migration 0002: Japanese and mixed-language search (RFC-014 §12).
--
-- The trigram FTS5 table is an additive second keyword index.
-- Every chunk is indexed in both unicode61 (chunk_fts, M1) and trigram
-- (chunk_fts_trigram, here). The two tables are never combined in a single
-- FTS query; the multilingual engine queries both and merges the results.
--
-- `trigram_fts_rowid` mirrors `fts_rowid` in keyword_index_records: it
-- stores the rowid for this chunk's row in chunk_fts_trigram, required
-- for contentless_delete=1 deletion.

ALTER TABLE keyword_index_records ADD COLUMN trigram_fts_rowid INTEGER;

-- Trigram FTS5 index (SQLite ≥ 3.43, bundled version 3.53.2).
-- Improves recall for:
--   - CJK scripts (Japanese kanji/kana runs)
--   - Partial-word matching
--   - Mixed Japanese-English terms
-- Larger index footprint than unicode61; measured in RFC-014 §18.
CREATE VIRTUAL TABLE chunk_fts_trigram USING fts5(
    title,
    heading_path,
    normalized_text,
    tokenize = 'trigram',
    content = '',
    contentless_delete = 1
);
