# Local Development

```sh
# Requires Rust 1.85+ (install via rustup.rs)

# Run all backend tests (non-GUI crates)
cargo test -p orbok-core -p orbok-db -p orbok-fs -p orbok-cache \
           -p orbok-extract -p orbok-models -p orbok-search \
           -p orbok-workers -p orbok-embed

# Run GUI + smoke tests
cargo test -p orbok-ui

# Run the GUI (default-members = ["crates/app"] so no -p needed)
cargo run

# Portable mode (data in ./orbok-data/ instead of platform app-data dir)
cargo run -- --portable

# Headless backend check (no display needed)
cargo run -- --check

# With a custom data dir
ORBOK_DATA_DIR=/tmp/orbok-dev cargo run -- --check
```

## Testing Philosophy

Tests validate design specifications (RFC acceptance criteria), not
merely the written code. Each crate's `tests.rs` cites the RFC
section it targets.

## Packaging

```sh
bash scripts/package.sh 1.0.0
# Produces orbok-1.0.0.tar.gz and orbok-1.0.0.tar.gz.sha256
```
