# Local Development

```sh
# Requires Rust 1.85+ (install via rustup.rs)
cargo test --workspace

# Headless backend check (no display needed)
ORBIT_DATA_DIR=/tmp/orbit-dev cargo run -p orbit-app -- --check

# Run the GUI
cargo run -p orbit-app
```

## Testing Philosophy

Tests validate design specifications (RFC acceptance criteria), not
merely the written code. Each crate's `tests.rs` cites the RFC
section it targets.
