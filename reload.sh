#!/bin/bash
# Restart AgentPulse without rebuilding (frontend changes only)
# Usage: ./reload.sh

set -e
cd "$(dirname "$0")"

echo "→ Killing running instance..."
pkill -f "claude-pulse" 2>/dev/null || true
sleep 0.5

# Try release binary first, fallback to debug
if [ -f "src-tauri/target/release/claude-pulse" ]; then
  BIN="src-tauri/target/release/claude-pulse"
elif [ -f "src-tauri/target/debug/claude-pulse" ]; then
  BIN="src-tauri/target/debug/claude-pulse"
else
  echo "Error: no binary found. Run ./dev.sh first."
  exit 1
fi

echo "→ Launching $BIN..."
"$BIN" &
echo "→ PID: $!"
