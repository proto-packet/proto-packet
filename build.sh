#!/usr/bin/env bash
set -euo pipefail

# Folders
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEP_RUST_DIR="$ROOT_DIR/dep/dep-rust"
COMPILE_DIR="$ROOT_DIR/compile"

# Runtime: Format, Lint & Test
cargo fmt --manifest-path "$DEP_RUST_DIR/Cargo.toml" --all
cargo clippy --manifest-path "$DEP_RUST_DIR/Cargo.toml" --all-targets --no-default-features -- -D warnings
cargo clippy --manifest-path "$DEP_RUST_DIR/Cargo.toml" --all-targets -- -D warnings
cargo clippy --manifest-path "$DEP_RUST_DIR/Cargo.toml" --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$DEP_RUST_DIR/Cargo.toml" --no-deps --all-features
cargo test --manifest-path "$DEP_RUST_DIR/Cargo.toml" --all-features

# Compiler: Format, Lint & Test
cargo fmt --manifest-path "$COMPILE_DIR/Cargo.toml" --all
cargo clippy --manifest-path "$COMPILE_DIR/Cargo.toml" --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$COMPILE_DIR/Cargo.toml" --workspace --no-deps
cargo test --manifest-path "$COMPILE_DIR/Cargo.toml" --workspace

# End-to-End Tests
"$ROOT_DIR/test/build.sh"
