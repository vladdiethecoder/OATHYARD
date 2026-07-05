#!/usr/bin/env bash
# Check if native renderer binary is stale compared to source files
set -euo pipefail

RENDERER_SRC="crates/oathyard_renderer/src/main.rs"
RENDERER_BIN="crates/oathyard_renderer/target/debug/oathyard-native-renderer"
RENDERER_WGSL="crates/oathyard_renderer/src/verdict_ring.wgsl"

if [[ ! -f "$RENDERER_BIN" ]]; then
    echo "⚠️  Renderer binary not found: $RENDERER_BIN"
    echo "   Building..."
    cargo build --manifest-path crates/oathyard_renderer/Cargo.toml
    exit 0
fi

BIN_MTIME=$(stat -c %Y "$RENDERER_BIN" 2>/dev/null || stat -f %m "$RENDERER_BIN")
SRC_MTIME=$(stat -c %Y "$RENDERER_SRC" 2>/dev/null || stat -f %m "$RENDERER_SRC")
WGSL_MTIME=$(stat -c %Y "$RENDERER_WGSL" 2>/dev/null || stat -f %m "$RENDERER_WGSL")

# Use the newest source file time
if [[ "$WGSL_MTIME" -gt "$SRC_MTIME" ]]; then
    NEWEST_SRC_MTIME=$WGSL_MTIME
else
    NEWEST_SRC_MTIME=$SRC_MTIME
fi

if [[ "$NEWEST_SRC_MTIME" -gt "$BIN_MTIME" ]]; then
    echo "⚠️  Renderer binary is stale (source modified $(date -d @$NEWEST_SRC_MTIME +%H:%M:%S), binary from $(date -d @$BIN_MTIME +%H:%M:%S))"
    echo "   Rebuilding..."
    cargo build --manifest-path crates/oathyard_renderer/Cargo.toml
fi

echo "✓ Renderer binary is up to date"
