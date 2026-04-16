# AgentPulse

A cross-platform desktop app that brings **Dynamic Island-inspired** real-time monitoring to your AI coding assistant sessions.

> Inspired by [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms) — a beautiful macOS-native app built with Swift/SwiftUI.
> AgentPulse is a cross-platform rewrite using [Tauri v2](https://tauri.app/) to support **Linux**, **Windows**, and **macOS**, extended with multi-provider support.

## Supported AI Coding Assistants

| Provider | Hook Events | Config Location |
|----------|-------------|-----------------|
| **Claude Code** | 8 events | `~/.claude/settings.json` |
| **Gemini CLI** | 9 events | `~/.gemini/settings.json` |
| **Codex CLI** (OpenAI) | 5 events | `~/.codex/hooks.json` + `config.toml` |
| **GitHub Copilot CLI** | 6 events | `~/.copilot/config.json` |

Provider icons from [@lobehub/icons](https://github.com/lobehub/lobe-icons).

### Hook Events per Provider

| Event (normalized) | Claude | Gemini | Codex | Copilot |
|---------------------|--------|--------|-------|---------|
| SessionStart | `SessionStart` | `BeforeAgent` | `SessionStart` | `sessionStart` |
| SessionEnd | `SessionEnd` | `AfterAgent` | — | `sessionEnd` |
| UserPromptSubmit | `UserPromptSubmit` | `BeforeModel` | `UserPromptSubmit` | `userPromptSubmitted` |
| PreToolUse | `PreToolUse` | `BeforeTool` | `PreToolUse` | `preToolUse` |
| PostToolUse | `PostToolUse` | `AfterTool` | `PostToolUse` | `postToolUse` |
| Stop | `Stop` | `AfterModel` | `Stop` | `agentStop` |
| PermissionRequest | `PermissionRequest` | — | — | — |
| PostToolUseFailure | `PostToolUseFailure` | — | — | — |
| Notification | — | `Notification` | — | `errorOccurred` |

All events normalized to PascalCase internally. Each provider's hook command sends JSON via `curl` to `http://localhost:{port}/hook/{provider}`.

### Field Name Normalization

Different CLIs use different JSON field names. AgentPulse auto-detects and normalizes:

| Internal Field | Accepted Aliases |
|---------------|-----------------|
| `session_id` | `session_id`, `sessionId`, `session` |
| `hook_event_name` | `hook_event_name`, `hookEventName`, `event`, `type` |
| `cwd` | `cwd`, `workingDirectory`, `projectDir` |
| `prompt` | `prompt`, `initialPrompt`, `input`, `message`, `userPrompt` |
| `tool_name` | `tool_name`, `toolName` |

If `session_id` is missing, default ID generated as `{provider}-default`.

### Session State Machine

```
SessionStart ──▶ Idle
                  │
    UserPromptSubmit / PreToolUse / PostToolUse
                  │
                  ▼
               Working ──Stop──▶ Idle (+ sound if enabled)
                  │
          PermissionRequest
                  │
                  ▼
           WaitingForUser ──PreToolUse──▶ Working
```

**Timeout-based transitions** (checked every 10 seconds):

| Condition | Action |
|-----------|--------|
| Active session, 30 sec no events | → Idle |
| Any session, 10 min no events | → Stale |
| Any session, 30 min no events | Removed from list |
| `SessionEnd` event received | Removed immediately |
| User clicks X button on session | Removed immediately |

**Hook → State mapping:**

| Hook Event | State Change |
|------------|-------------|
| `SessionStart` | → Idle (new session created) |
| `UserPromptSubmit` | → Working |
| `PreToolUse` / `PostToolUse` / `PostToolUseFailure` | → Working |
| `PermissionRequest` | → WaitingForUser |
| `Stop` | → Idle (triggers completion sound if was Working) |
| `SessionEnd` | Session removed from list |

**Status indicator colors** (fixed, not accent-dependent):

| State | Dot | Label Color |
|-------|-----|-------------|
| Working | 🟢 Green (`#4ade80`) | Green |
| Waiting | 🟠 Orange (`#ffb340`) | Orange |
| Idle | ⚪ Gray | — |
| Stale | 🔘 Dim gray | Gray |

## Sound System

External MP3/WAV/OGG files in `~/.config/agentpulse/sounds/`. Each provider can have its own completion sound.

### Setup

1. Open Settings → enable **Sound on Complete**
2. **Per-Provider Sounds** section shows one dropdown per provider
3. Click 📁 to open the sounds folder
4. Drop your MP3/WAV/OGG files there
5. Files appear in the dropdowns
6. Click ▶ next to each provider to preview

### Auto-matching

If a sound file starts with the provider ID (e.g., `claude.mp3`, `gemini.wav`), AgentPulse auto-assigns it on first launch.

### Generate TTS sounds (optional)

Generate Chinese voice notifications using [edge-tts](https://github.com/rany2/edge-tts):

```bash
pip install edge-tts
mkdir -p ~/.config/agentpulse/sounds

for p in claude gemini copilot codex; do
  edge-tts --voice "zh-TW-HsiaoChenNeural" \
           --text "${p}任務完成" \
           --write-media ~/.config/agentpulse/sounds/${p}.mp3
done
```

Audio playback uses [`rodio`](https://github.com/RustAudio/rodio) (Rust-side, no browser CSP issues).

## Features

- **Dynamic Island Style** — Floating capsule expands on hover
- **Multi-Provider** — Claude, Gemini, Codex, Copilot simultaneously
- **Provider Icons** — Each session shows provider's official icon
- **Status Dot** — Inline colored indicator (green/orange/gray)
- **3-Line Session Info** — Project name + status, working directory, last prompt (italic)
- **Click to Focus** — Brings session's terminal window to foreground via stored window ID
- **Remove Session** — X button on hover, red on hover, click to remove
- **Smart Re-render** — Timer updates in-place, only structural changes trigger full re-render (preserves hover state)
- **Per-Provider Sounds** — Each CLI plays its own sound on completion
- **Bounce Animation** — Window bounces when collapsing
- **Draggable** — Drag capsule anywhere
- **System Tray** — Show/hide and quit
- **Settings** — Providers, accent color (5), text size (S/M/L), per-provider sounds, pin expanded
- **Config File** — All settings in `~/.config/agentpulse/config.json` (zero localStorage)
- **Auto-Detection** — First launch detects installed CLIs via `which`
- **Open Settings File** — Button per provider opens its config file in default editor
- **Cross-Platform** — Linux, Windows, macOS (Tauri v2)

## Install

### Linux — AppImage (no install needed)

```bash
chmod +x AgentPulse_0.1.0_amd64.AppImage
./AgentPulse_0.1.0_amd64.AppImage
```

### Linux — .deb (Ubuntu / Debian)

```bash
sudo dpkg -i AgentPulse_0.1.0_amd64.deb
claude-pulse  # binary name
```

### Linux — .rpm (Fedora / RHEL)

```bash
sudo rpm -i AgentPulse-0.1.0-1.x86_64.rpm
claude-pulse
```

### Windows / macOS

Download from [Releases](../../releases/latest).

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/): `cargo install tauri-cli`
- **Linux** dependencies:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
  ```
  (`libasound2-dev` needed by rodio for audio)

### Build

```bash
git clone https://github.com/yazelin/AgentPulse.git
cd AgentPulse
cargo tauri build
```

Output:
```
src-tauri/target/release/claude-pulse              # binary
src-tauri/target/release/bundle/deb/               # .deb
src-tauri/target/release/bundle/rpm/               # .rpm
src-tauri/target/release/bundle/appimage/          # .AppImage
```

### Development Workflow

Use the included scripts for fast iteration:

```bash
./dev.sh           # debug build + run (fast compile)
./dev.sh release   # release build + run (slower compile, faster runtime)
./reload.sh        # restart without rebuilding (frontend changes only)
```

For full Tauri dev mode (auto-reload):
```bash
cargo tauri dev
```

Frontend is static HTML/CSS/JS — no bundler needed. Frontend changes require restart only (no rebuild).

## Usage

### First Launch

1. AgentPulse opens with Settings showing detected providers
2. Check the providers you want to monitor (Claude, Gemini, Codex, Copilot)
3. Hooks are automatically installed into each provider's config file
4. (Optional) Open sounds folder, add MP3 files, set per-provider sounds
5. Close settings — the capsule is ready

### Controls

| Action | Effect |
|--------|--------|
| **Hover** capsule | Expand session list |
| **Move away** | Collapse (with bounce animation) |
| **Drag** capsule | Reposition anywhere on screen |
| **Click** session row | Focus its terminal window |
| **Hover** session row | Show remove (X) button |
| **Click** X button | Remove session from list |
| **Pin** button | Keep panel expanded without hovering |
| **Gear** button | Open settings |
| **Tray** → Show/Hide | Toggle visibility (positions at current monitor top-center) |
| **Tray** → Quit | Exit AgentPulse |

### Settings

| Setting | Options |
|---------|---------|
| **Providers** | Enable/disable each CLI; install/remove hooks; auto-detected |
| **Open settings file** | Button per provider opens its config file in editor |
| **Keep Expanded** | Pin panel open without hovering |
| **Sound on Complete** | Play sound when AI finishes |
| **Per-Provider Sounds** | Each CLI gets own sound; ▶ to preview; 📁 to add files |
| **Accent Color** | Purple, Cyan, Green, Orange, Pink |
| **Size** | S / M / L text scaling |

## Architecture

### How It Works

```
Claude Code ──curl + X-Window-Id header──▶
Gemini CLI  ──curl + X-Window-Id header──▶  AgentPulse HTTP Server
Codex CLI   ──curl + X-Window-Id header──▶  (localhost:19280-19289)
Copilot CLI ──curl + X-Window-Id header──▶
                                                    │
                                                    ▼
                                          Session Manager
                                          (state machine, timers)
                                                    │
                                                    ▼
                                          Capsule UI (1s polling)
```

1. AgentPulse starts a TCP server on port 19280-19289
2. Port written to `~/.agentpulse/port` for CLI hooks
3. On provider enable, hooks written to each CLI's config file (auto-clean before install)
4. Each CLI sends JSON events via `curl` with `X-Window-Id` header (terminal window ID via process tree walk)
5. Server parses URL `/hook/{provider}` to identify provider
6. Field name normalization handles different CLIs
7. Event names normalized to PascalCase
8. Session manager updates state machine
9. UI polls state every 1 second; smart re-render only on structural changes

### Window ID Capture (Click-to-Focus)

The curl hook walks up the parent process tree to find the terminal window:

```bash
$(p=$PPID; w=""; while [ "$p" -gt 1 ]; do
  w=$(xdotool search --pid $p 2>/dev/null | head -1);
  [ -n "$w" ] && break;
  p=$(awk '{print $4}' /proc/$p/stat 2>/dev/null);
  [ -z "$p" ] && break;
done; echo "$w")
```

The found window ID is sent as `X-Window-Id` header. Click on session → `xdotool windowactivate <stored_wid>`.

### Hook Installation Details

**Claude Code** (`~/.claude/settings.json`):
```json
{
  "hooks": {
    "SessionStart": [{ "matcher": "", "hooks": [{ "type": "command", "command": "curl -H 'X-Window-Id: ...' ... /hook/claude", "async": true }] }]
  }
}
```

**Gemini CLI** (`~/.gemini/settings.json`):
```json
{
  "hooks": {
    "BeforeAgent": [{ "matcher": "", "hooks": [{ "type": "command", "command": "curl ... /hook/gemini", "async": true }] }]
  }
}
```

**Codex CLI** (`~/.codex/hooks.json` + enables `codex_hooks` feature in `config.toml`):
```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "curl ... /hook/codex" }] }]
  }
}
```

**GitHub Copilot CLI** (`~/.copilot/config.json` — uses `bash` field):
```json
{
  "hooks": {
    "sessionStart": [{ "type": "command", "bash": "curl ... /hook/copilot" }]
  }
}
```

### Config File

`~/.config/agentpulse/config.json`:
```json
{
  "setup_done": true,
  "appearance": {
    "accent_color": "purple",
    "text_size": "medium",
    "pin_expanded": false,
    "sound_enabled": true,
    "provider_sounds": {
      "claude": "claude.mp3",
      "gemini": "gemini.mp3",
      "codex": "codex.mp3",
      "copilot": "copilot.mp3"
    }
  },
  "providers": {
    "claude": { "enabled": true, "name": "Claude Code", "settings_path": "~/.claude/settings.json" },
    "gemini": { "enabled": true, "name": "Gemini CLI", "settings_path": "~/.gemini/settings.json" },
    "codex": { "enabled": false, "name": "Codex CLI", "settings_path": "~/.codex/hooks.json" },
    "copilot": { "enabled": false, "name": "GitHub Copilot", "settings_path": "~/.copilot/config.json" }
  }
}
```

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | Tauri v2 |
| Backend | Rust (tokio, serde, chrono, rodio) |
| Frontend | HTML / CSS / JS (no framework, no bundler) |
| HTTP Server | tokio TCP (raw HTTP parsing) |
| Window | WebKitGTK (Linux), WebView2 (Windows), WKWebView (macOS) |
| Audio | rodio (Rust audio playback) |
| Icons | [@lobehub/icons](https://github.com/lobehub/lobe-icons) (inline SVG) |
| Linux extras | `webkit2gtk`, `gtk`, `gdk` crates for window management; `xdotool` for window focus |

## Project Structure

```
AgentPulse/
├── src/                           # Frontend
│   ├── index.html                 # Capsule, expanded, settings views
│   ├── styles.css                 # All styles (no CSS transitions for X11 compat)
│   └── main.js                    # Tauri IPC, state, UI, provider icons, sound playback
├── src-tauri/                     # Backend (Rust)
│   ├── Cargo.toml                 # Dependencies (tauri, rodio, webkit2gtk, etc.)
│   ├── tauri.conf.json            # Window, tray, bundle config
│   ├── capabilities/default.json  # Tauri v2 permissions
│   └── src/
│       ├── lib.rs                 # App setup, tray, window mgmt, all Tauri commands
│       ├── config.rs              # Config R/W, provider detection, sounds dir
│       ├── hook_server.rs         # HTTP server, URL routing, X-Window-Id parsing
│       ├── hook_event.rs          # RawHookEvent (field aliases) → HookEvent (normalized)
│       ├── session.rs             # State machine, SessionManager, AppState
│       └── hooks_configurator.rs  # Per-provider hook install/remove
├── dev.sh                         # Quick build + run script
├── reload.sh                      # Restart without rebuild
├── package.json
├── README.md
└── LICENSE
```

## Known Limitations (Linux / X11)

| Issue | Workaround |
|-------|------------|
| `rgba()` backgrounds ghost on transparent windows | Opaque `rgb()` backgrounds; transparent window only for rounded corners |
| CSS `-webkit-app-region: drag` doesn't work | Tauri `startDragging()` API via JS `mousedown` |
| `mouseleave` unreliable on transparent windows | Tauri `cursor-left` event + CSS `:hover` polling |
| CSS `:hover` pseudo-class unreliable for style changes | JS `mouseenter`/`mouseleave` with inline `style.color` |
| `<select>` dropdown uses system native styling | Custom div-based dropdown |
| CSS `transition` / `animation` causes pixel ghosting | All transitions removed; bounce via Rust `set_position` thread |
| `transform: translateZ(0)` creates black compositing layers | Not used |
| DOM re-render destroys hover state | Smart re-render: structural changes only; timers in-place |
| Browser CSP blocks blob URLs and local files | Audio plays via Rust `rodio` (no browser audio at all) |

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms)
- Provider icons from [@lobehub/icons](https://github.com/lobehub/lobe-icons)
- Audio playback via [rodio](https://github.com/RustAudio/rodio)
- TTS sound generation example via [edge-tts](https://github.com/rany2/edge-tts)

## License

MIT
