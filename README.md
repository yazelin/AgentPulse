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

## Features

- **Dynamic Island Style** — A compact capsule UI floats above your screen, expanding on hover
- **Multi-Provider** — Monitor Claude, Gemini, Codex, and Copilot sessions simultaneously
- **Provider Icons** — Each session shows its provider's official icon (via [@lobehub/icons](https://github.com/lobehub/lobe-icons))
- **Real-time Tracking** — Working, Waiting, Idle, Stale states with live timer
- **3-Line Session Info** — Project name, working directory (`~/path`), and last prompt (italic)
- **Click to Focus** — Click a session to bring its terminal window to foreground (Linux: `xdotool`)
- **Bounce Animation** — Satisfying bounce effect when panel collapses
- **Draggable** — Drag the capsule anywhere on screen
- **System Tray** — Show/hide panel and quit from tray
- **Settings** — Accent color (5), text size (S/M/L), sound on complete (5 sounds), pin expanded
- **Config File** — All settings in `~/.config/agentpulse/config.json` (no localStorage)
- **Zero Config Start** — First launch opens settings with auto-detected providers
- **Cross-Platform** — Linux, Windows, macOS (Tauri v2)

## Session States

| State | Description |
|-------|-------------|
| **Working** | AI is processing (purple/accent icon) |
| **Waiting** | Waiting for user input or approval (orange icon) |
| **Idle** | Session is idle (gray icon) |
| **Stale** | No activity for 10+ minutes (dim icon, removed after 30 min) |

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
| **Move away** | Collapse (with bounce) |
| **Drag** capsule | Reposition anywhere |
| **Click** session | Focus its terminal window |
| **Pin** button | Keep panel expanded |
| **Gear** button | Open settings |
| **Tray** → Show/Hide | Toggle visibility |
| **Tray** → Quit | Exit |

### Settings

| Setting | Options |
|---------|---------|
| **Providers** | Enable/disable each CLI with checkbox |
| **Keep Expanded** | Pin panel open without hovering |
| **Sound on Complete** | Glass, Ping, Pop, Chime, Bell |
| **Accent Color** | Purple, Cyan, Green, Orange, Pink |
| **Size** | S / M / L text scaling |

## Architecture

### How It Works

```
Claude Code ──curl──▶
Gemini CLI  ──curl──▶  AgentPulse HTTP Server  ──▶  Capsule UI
Codex CLI   ──curl──▶  (localhost:19280)            (Tauri window)
Copilot CLI ──curl──▶
```

1. AgentPulse starts a TCP server on port 19280-19289
2. On provider enable, hooks are written to each CLI's config file
3. Each CLI sends events via `curl` to `/hook/{provider}` (e.g., `/hook/claude`, `/hook/gemini`)
4. Events are normalized to common names and routed to the session manager
5. UI updates in real-time via 1-second polling

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

**GitHub Copilot CLI** (`~/.copilot/config.json`):
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
│   ├── styles.css                 # All styles (no transitions for X11 compat)
│   └── main.js                    # All logic: Tauri IPC, state, UI, provider icons
├── src-tauri/                     # Backend
│   ├── Cargo.toml                 # Rust dependencies
│   ├── tauri.conf.json            # Tauri config (window, tray, bundle)
│   ├── capabilities/default.json  # Tauri v2 permissions
│   └── src/
│       ├── lib.rs                 # App setup, tray, window mgmt, Tauri commands
│       ├── config.rs              # Config file R/W, provider detection
│       ├── hook_server.rs         # HTTP server, URL routing, event normalization
│       ├── hook_event.rs          # Event data model (provider, session_id, etc.)
│       ├── session.rs             # Session state machine, manager, AppState
│       └── hooks_configurator.rs  # Per-provider hook installation (4 formats)
├── package.json
├── README.md
└── LICENSE
```

## Known Limitations (Linux / X11)

| Issue | Workaround |
|-------|------------|
| `rgba()` backgrounds ghost on transparent windows | Use opaque `rgb()` backgrounds; transparent window only for rounded corners |
| CSS `-webkit-app-region: drag` doesn't work | Tauri `startDragging()` API |
| `mouseleave` unreliable on transparent windows | Tauri cursor-left event + CSS `:hover` polling |
| `<select>` uses system native styling | Custom div-based dropdown |
| CSS `transition` / `animation` causes ghosting | All transitions removed; bounce via Rust `set_position` |
| `transform: translateZ(0)` creates black compositing layers | Not used |

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms)
- Provider icons from [@lobehub/icons](https://github.com/lobehub/lobe-icons)

## License

MIT
