# FAQ

**Does orbok upload my files?**
No. All processing is local. Even with an embedding model installed,
inference runs on your computer only.

**Can I search encrypted files?**
Not in v0.x. Encrypted files are skipped with an `unsupported_format` error.

**Why is semantic search unavailable?**
No embedding model is registered. Use the Models view to locate or
install one. Keyword search always works without a model.

**How do I free up disk space?**
Open the Storage view and run Safe Cleanup. For more space, delete the
vector index (it will be rebuilt from your source files).

**My source is showing as Stale. What does that mean?**
The source file changed after it was indexed. orbok will reindex it on
the next scan. You can trigger a rescan manually from the Indexing view.

**How do I search Japanese text?**
Just type normally. orbok detects CJK characters and uses the trigram
index automatically. Full-width characters are normalized.

**Can I use orbok on a server without a display?**
Yes. Run `orbok --check` to validate the backend. Use the orbok-workers
library crate to drive indexing and search programmatically.
