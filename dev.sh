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
pkill -9 -x agent-pulse 2>/dev/null || true
sleep 1

if [ "$MODE" = "release" ]; then
  echo "→ Building release binary (cargo tauri build)..."
  # IMPORTANT: must use `cargo tauri build`, NOT `cargo build --release`.
  # Plain cargo build skips frontend embedding, so the webview falls back to
  # devUrl (localhost:1420) and shows "Could not connect to localhost".
  npm run build -- --no-bundle
  BIN="src-tauri/target/release/agent-pulse"
else
  echo "→ Building debug binary..."
  cargo build --manifest-path src-tauri/Cargo.toml
  BIN="src-tauri/target/debug/agent-pulse"
fi

echo "→ Launching $BIN..."
# GDK_BACKEND=x11 keeps always-on-top working on GNOME Wayland (XWayland path)
GDK_BACKEND=x11 "$BIN" &
echo "→ PID: $!"
echo "Done."
