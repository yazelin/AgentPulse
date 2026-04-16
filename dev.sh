#!/bin/bash
# Build and run AgentPulse for testing
# Usage: ./dev.sh [release]
#   release - build optimized binary (slower compile, faster run)
#   default - debug build (fast compile)

set -e
cd "$(dirname "$0")"

MODE="${1:-debug}"

echo "→ Killing any running instance..."
pkill -f "claude-pulse" 2>/dev/null || true
sleep 0.5

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
echo "Done. App is running."
