#!/usr/bin/env bash
set -euo pipefail

# Folders
PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IDE_DIR="${1:-/Applications/RustRover.app}"

# Build & Verify
JAVA_HOME="$IDE_DIR/Contents/jbr/Contents/Home" "$PLUGIN_DIR/gradlew" --project-dir "$PLUGIN_DIR" \
    -Pide="$IDE_DIR" buildPlugin verifyPlugin
