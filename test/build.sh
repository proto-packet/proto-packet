#!/usr/bin/env bash
set -euo pipefail

# Folders
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$TEST_DIR/.." && pwd)"
RUST_DIR="$TEST_DIR/test-rust"

# Compile Schema
cargo run --quiet --manifest-path "$ROOT_DIR/compile/Cargo.toml" \
    -p proto-packet-cli \
    -- compile rust "$TEST_DIR/schema" "$RUST_DIR/proto-packet-test/src"

# Format, Lint & Test
cargo fmt --manifest-path "$RUST_DIR/Cargo.toml" --all
cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" --workspace --all-targets -- -D warnings
cargo test --manifest-path "$RUST_DIR/Cargo.toml" --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$RUST_DIR/Cargo.toml" --workspace --no-deps
