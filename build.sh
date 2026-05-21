#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo fmt    --manifest-path "$ROOT_DIR/dep/dep-rust/Cargo.toml"
cargo test   --manifest-path "$ROOT_DIR/dep/dep-rust/Cargo.toml"
cargo clippy --manifest-path "$ROOT_DIR/dep/dep-rust/Cargo.toml" --all-targets -- -D warnings

cargo fmt    --manifest-path "$ROOT_DIR/compile/Cargo.toml" --all
cargo test   --manifest-path "$ROOT_DIR/compile/Cargo.toml" --all-features
cargo clippy --manifest-path "$ROOT_DIR/compile/Cargo.toml" --all-features --all-targets -- -D warnings
