# Local Development

```sh
# Requires Rust 1.85+ (install via rustup.rs)

# Run the full test suite
cargo test --workspace --lib

# Run the GUI
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

Test organisation mirrors the module structure:

- Inline tests live in `src/tests.rs`.
- If `tests.rs` exceeds ~300 ELOC, contents move into submodules
  under `src/tests/`.
- The same line-count rules apply inside `tests/`.

## Module Style

orbok uses Rust 2018+ module layout throughout:

- A `foo.rs` file and a `foo/` subdirectory may coexist.
- `mod.rs` is never used. Place the module router in `foo.rs` and
  submodule files inside `foo/`.

## Packaging

```sh
bash scripts/package.sh 0.17.0
# Produces dist/orbok-v0.17.0.tar.gz and dist/orbok-v0.17.0.tar.gz.sha256
```

The archive is flat (no parent directory). Files unpack directly into
the extraction destination.
