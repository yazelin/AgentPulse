# AgentPulse

A cross-platform desktop app that brings **Dynamic Island-inspired** real-time monitoring to your AI coding assistant sessions.

> Inspired by [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms) — a beautiful macOS-native app built with Swift/SwiftUI.
> AgentPulse is a cross-platform rewrite using [Tauri v2](https://tauri.app/) to support **Linux**, **Windows**, and **macOS**, extended with multi-provider support.

## Supported AI Coding Assistants

| Provider | Icon Source | Hook Events | Config Location |
|----------|-----------|-------------|-----------------|
| **Claude Code** | [@lobehub/icons](https://github.com/lobehub/lobe-icons) | 8 events | `~/.claude/settings.json` |
| **Gemini CLI** | @lobehub/icons | 9 events | `~/.gemini/settings.json` |
| **Codex CLI** (OpenAI) | @lobehub/icons | 5 events | `~/.codex/hooks.json` + `config.toml` |
| **GitHub Copilot CLI** | @lobehub/icons | 6 events | `~/.copilot/config.json` |

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

All events are normalized to PascalCase internally. Each provider's hook command sends JSON via `curl` to `http://localhost:{port}/hook/{provider}`.

### Field Name Normalization

Different CLIs use different JSON field names. AgentPulse auto-detects and normalizes them:

| Internal Field | Accepted Aliases |
|---------------|-----------------|
| `session_id` | `session_id`, `sessionId`, `session` |
| `hook_event_name` | `hook_event_name`, `hookEventName`, `event`, `type` |
| `cwd` | `cwd`, `workingDirectory`, `projectDir` |
| `prompt` | `prompt`, `initialPrompt`, `input`, `message`, `userPrompt` |
| `tool_name` | `tool_name`, `toolName` |

If `session_id` is missing, a default ID is generated as `{provider}-default`.

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

**Status indicator colors** (fixed, not affected by accent color):

| State | Dot Color | Label Color |
|-------|-----------|-------------|
| Working | Green (`#4ade80`) | Green |
| Waiting | Orange (`#ffb340`) | Orange |
| Idle | Gray (25% white) | — |
| Stale | Dim gray (10% white) | Gray |

## Features

- **Dynamic Island Style** — A compact capsule UI floats above your screen, expanding on hover
- **Multi-Provider** — Monitor Claude, Gemini, Codex, and Copilot sessions simultaneously
- **Provider Icons** — Each session shows its provider's official icon (via [@lobehub/icons](https://github.com/lobehub/lobe-icons))
- **Status Dot** — Inline colored indicator (green/orange/gray) next to project name
- **Real-time Tracking** — Working, Waiting, Idle, Stale states with live timer
- **3-Line Session Info** — Project name + status, working directory (`~/path`), last prompt (italic)
- **Click to Focus** — Click a session to bring its terminal window to foreground (searches terminal emulator windows first)
- **Remove Session** — X button appears on hover, turns red on hover, click to remove
- **Smart Re-render** — Session list only re-renders on structural changes; timers update in-place preserving hover state
- **Bounce Animation** — Satisfying bounce effect when panel collapses
- **Draggable** — Drag the capsule anywhere on screen
- **System Tray** — Show/hide panel and quit from tray
- **Settings** — Providers, accent color (5), text size (S/M/L), sound on complete (5 sounds), pin expanded
- **Config File** — All settings in `~/.config/agentpulse/config.json` (zero localStorage)
- **Auto-Detection** — First launch detects installed CLIs via `which` and config directory presence
- **Cross-Platform** — Linux, Windows, macOS (Tauri v2)

## Session States

| State | Dot | Description |
|-------|-----|-------------|
| **Working** | 🟢 | AI is processing |
| **Waiting** | 🟠 | Waiting for user input or approval |
| **Idle** | ⚪ | Session is idle |
| **Stale** | 🔘 | No activity for 10+ minutes (removed after 30 min) |

## Install

### Linux — AppImage (no install needed)

```bash
chmod +x AgentPulse_0.1.0_amd64.AppImage
./AgentPulse_0.1.0_amd64.AppImage
```

### Linux — .deb (Ubuntu / Debian)

```bash
sudo dpkg -i AgentPulse_0.1.0_amd64.deb
claude-pulse  # binary name (will be renamed to agent-pulse in future release)
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
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
  ```

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
src-tauri/target/release/bundle/appimage/           # .AppImage
```

### Development

```bash
cargo tauri dev
```

Frontend is static HTML/CSS/JS — no bundler needed. Changes to `src/` take effect on restart.

## Usage

### First Launch

1. AgentPulse opens with Settings showing detected providers
2. Check the providers you want to monitor (Claude, Gemini, Codex, Copilot)
3. Hooks are automatically installed into each provider's config file
4. Close settings — the capsule is ready

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
| **Providers** | Enable/disable each CLI with checkbox; auto-detects installed CLIs |
| **Keep Expanded** | Pin panel open without hovering |
| **Sound on Complete** | Glass, Ping, Pop, Chime, Bell (Web Audio synthesis) |
| **Accent Color** | Purple, Cyan, Green, Orange, Pink |
| **Size** | S / M / L text scaling |

## Architecture

### How It Works

```
Claude Code ──curl──▶
Gemini CLI  ──curl──▶  AgentPulse HTTP Server  ──▶  Capsule UI
Codex CLI   ──curl──▶  (localhost:19280-19289)      (Tauri window)
Copilot CLI ──curl──▶
```

1. AgentPulse starts a TCP server on port 19280-19289 (tries each until one is available)
2. Port is written to `~/.agentpulse/port` for CLI hooks to read
3. On provider enable, hooks are written to each CLI's config file
4. Each CLI sends JSON events via `curl` to `/hook/{provider}`
5. Raw JSON is parsed with field name normalization (handles different CLIs)
6. Events are normalized to common PascalCase names
7. Session manager updates state machine
8. UI polls state every 1 second; smart re-render only on structural changes

### Hook Installation Details

**Claude Code** (`~/.claude/settings.json`):
```json
{
  "hooks": {
    "SessionStart": [{ "matcher": "", "hooks": [{ "type": "command", "command": "curl ... /hook/claude", "async": true }] }]
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

**GitHub Copilot CLI** (`~/.copilot/config.json` — uses `bash` field instead of `command`):
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
    "sound_enabled": false,
    "sound_name": "glass"
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
| Backend | Rust (tokio, serde, chrono) |
| Frontend | HTML / CSS / JS (no framework, no bundler) |
| HTTP Server | tokio TCP (raw HTTP parsing) |
| Window | WebKitGTK (Linux), WebView2 (Windows), WKWebView (macOS) |
| Icons | [@lobehub/icons](https://github.com/lobehub/lobe-icons) (inline SVG) |
| Linux extras | `webkit2gtk`, `gtk`, `gdk` crates for window management |

## Project Structure

```
AgentPulse/
├── src/                           # Frontend
│   ├── index.html                 # Main HTML (capsule, expanded, settings views)
│   ├── styles.css                 # All styles (no CSS transitions for X11 compat)
│   └── main.js                    # All logic: Tauri IPC, state, UI, provider icons
├── src-tauri/                     # Backend (Rust)
│   ├── Cargo.toml                 # Dependencies
│   ├── tauri.conf.json            # Tauri config (window, tray, bundle)
│   ├── capabilities/default.json  # Tauri v2 permissions
│   └── src/
│       ├── lib.rs                 # App setup, tray, window mgmt, Tauri commands
│       ├── config.rs              # Config file R/W, provider detection, path expansion
│       ├── hook_server.rs         # HTTP server, URL-based provider routing, event normalization
│       ├── hook_event.rs          # RawHookEvent (field aliases) → HookEvent (normalized)
│       ├── session.rs             # Session state machine, SessionManager, AppState
│       └── hooks_configurator.rs  # Per-provider hook installation (4 different formats)
├── package.json
├── README.md
└── LICENSE
```

## Known Limitations (Linux / X11)

| Issue | Workaround |
|-------|------------|
| `rgba()` backgrounds ghost on transparent windows | Use opaque `rgb()` backgrounds; transparent window only for rounded corners |
| CSS `-webkit-app-region: drag` doesn't work | Tauri `startDragging()` API via JS `mousedown` |
| `mouseleave` unreliable on transparent windows | Tauri `cursor-left` event + CSS `:hover` polling fallback |
| CSS `:hover` pseudo-class unreliable for style changes | JS `mouseenter`/`mouseleave` with inline `style.color` |
| `<select>` dropdown uses system native styling | Custom div-based dropdown |
| CSS `transition` / `animation` causes pixel ghosting | All transitions removed; bounce via Rust `set_position` thread |
| `transform: translateZ(0)` creates black compositing layers | Not used |
| DOM re-render destroys hover state | Smart re-render: only structural changes trigger full re-render; timers update in-place |

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms)
- Provider icons from [@lobehub/icons](https://github.com/lobehub/lobe-icons)

## License

MIT
