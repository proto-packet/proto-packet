#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo fmt    --manifest-path "$ROOT_DIR/dep/dep-rust/Cargo.toml"
cargo test   --manifest-path "$ROOT_DIR/dep/dep-rust/Cargo.toml"

cargo fmt    --manifest-path "$ROOT_DIR/compile/Cargo.toml" --all
cargo test   --manifest-path "$ROOT_DIR/compile/Cargo.toml" --all-features

"$ROOT_DIR/test/build.sh"
