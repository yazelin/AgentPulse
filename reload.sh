#!/bin/bash
# Restart AgentPulse without rebuilding.
# NOTE: Frontend files (src/*) are embedded in the binary at build time.
#       If you changed src/, use ./dev.sh instead to rebuild.
# This script only restarts the existing binary — useful for testing config
# reset, session cleanup, or startup flow.

set -e
cd "$(dirname "$0")"

echo "→ Killing running instance..."
pkill -9 -x agent-pulse 2>/dev/null || true
sleep 1

if [ -f "src-tauri/target/release/agent-pulse" ]; then
  BIN="src-tauri/target/release/agent-pulse"
elif [ -f "src-tauri/target/debug/agent-pulse" ]; then
  BIN="src-tauri/target/debug/agent-pulse"
else
  echo "Error: no binary found. Run ./dev.sh first."
  exit 1
fi

echo "→ Launching $BIN..."
"$BIN" &
echo "→ PID: $!"
