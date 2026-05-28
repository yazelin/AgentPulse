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

# Resolve a Tauri CLI: prefer the local npm one (@tauri-apps/cli), else cargo-tauri.
# Lets the script work whether or not `npm install` has been run on this machine.
if [ -x node_modules/.bin/tauri ]; then
  TAURI="npm run tauri --"
elif command -v cargo-tauri >/dev/null 2>&1; then
  TAURI="cargo tauri"
else
  echo "✗ No Tauri CLI found. Run 'npm install' (gets @tauri-apps/cli) or 'cargo install tauri-cli'." >&2
  exit 1
fi

echo "→ Killing any running instance..."
pkill -9 -x agent-pulse 2>/dev/null || true
sleep 1

if [ "$MODE" = "release" ]; then
  echo "→ Building release binary (cargo tauri build)..."
  # IMPORTANT: must use `cargo tauri build`, NOT `cargo build --release`.
  # Plain cargo build skips frontend embedding, so the webview falls back to
  # devUrl (localhost:1420) and shows "Could not connect to localhost".
  $TAURI build --no-bundle
  BIN="src-tauri/target/release/agent-pulse"
else
  echo "→ Building debug binary..."
  cargo build --manifest-path src-tauri/Cargo.toml
  BIN="src-tauri/target/debug/agent-pulse"
fi

echo "→ Launching $BIN..."
# Default: native Wayland (avoids XWayland transparent-window ghosting on GNOME).
# If the capsule sinks behind other windows on your Wayland compositor, opt into
# the XWayland path with AGENTPULSE_GDK_X11=1 (forces always-on-top, but can ghost).
[ -n "$AGENTPULSE_GDK_X11" ] && export GDK_BACKEND=x11
"$BIN" &
echo "→ PID: $!"
echo "Done."
