#!/bin/bash
# Build and run AgentPulse for testing (any changes — Rust or frontend)
# Frontend files are embedded into the binary at build time, so ANY change needs a rebuild.
#
# Usage:
#   ./dev.sh           # debug build (fast compile, slower runtime)
#   ./dev.sh release   # release build (slower compile, faster runtime)

set -e
cd "$(dirname "$0")"

MODE="${1:-debug}"

echo "→ Killing any running instance..."
pkill -9 -f "claude-pulse" 2>/dev/null || true
sleep 1

if [ "$MODE" = "release" ]; then
  echo "→ Building release binary..."
  cargo build --release --manifest-path src-tauri/Cargo.toml
  BIN="src-tauri/target/release/claude-pulse"
else
  echo "→ Building debug binary..."
  cargo build --manifest-path src-tauri/Cargo.toml
  BIN="src-tauri/target/debug/claude-pulse"
fi

echo "→ Launching $BIN..."
"$BIN" &
echo "→ PID: $!"
echo "Done."
