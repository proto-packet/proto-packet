#!/usr/bin/env bash
set -euo pipefail

TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$TEST_DIR/.." && pwd)"
SCHEMA_DIR="$TEST_DIR/schema"
TARGET_DIR="$TEST_DIR/test-rust/src"

cargo run --quiet --manifest-path "$ROOT_DIR/compile/Cargo.toml" --bin proto-packet-cli -- \
    compile rust "$SCHEMA_DIR" "$TARGET_DIR"

cargo test --manifest-path "$TEST_DIR/test-rust/Cargo.toml"
