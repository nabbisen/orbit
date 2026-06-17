# Local Development

```sh
# Requires Rust 1.85+ (install via rustup.rs)
cargo test --workspace

# Headless backend check (no display needed)
ORBIT_DATA_DIR=/tmp/orbok-dev cargo run -p orbok-app -- --check

# Run the GUI
cargo run -p orbok-app
```

## Testing Philosophy

Tests validate design specifications (RFC acceptance criteria), not
merely the written code. Each crate's `tests.rs` cites the RFC
section it targets.
