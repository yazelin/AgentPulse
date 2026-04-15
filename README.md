# AgentPulse

A cross-platform desktop app that brings **Dynamic Island-inspired** real-time monitoring to your AI coding assistant sessions (Claude Code, Gemini CLI, GitHub Copilot CLI, Codex CLI).

> Inspired by [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms) — a beautiful macOS-native app built with Swift/SwiftUI.
> AgentPulse is a cross-platform rewrite using [Tauri v2](https://tauri.app/) to support **Linux**, **Windows**, and **macOS**, with planned support for multiple AI coding assistants.

## Features

- **Dynamic Island Style** — A compact capsule UI floats above your screen, expanding on hover to show full session details
- **Real-time Session Tracking** — Monitor multiple sessions simultaneously with working, waiting, or idle status
- **Multi-session View** — Expanded panel shows project name, working directory, and last prompt for each session
- **Click to Focus** — Click a session to bring its terminal window to the foreground
- **System Tray** — Show/hide panel and quit from the system tray
- **Draggable** — Drag the capsule anywhere on screen
- **Bounce Animation** — Satisfying bounce effect when the panel collapses
- **Customizable** — Accent color (5 colors), text size (S/M/L), sound notifications, pin expanded
- **Fully Local** — All data stays on localhost via hooks. Nothing leaves your machine
- **Zero Configuration** — Automatically sets up hooks on first launch
- **Cross-Platform** — Linux, Windows, macOS

## Session States

| State | Description |
|-------|-------------|
| **Working** | AI is processing |
| **Waiting** | Waiting for user input or approval |
| **Idle** | Session is idle |
| **Stale** | No activity for over 10 minutes |

## Install

### Linux — AppImage (no install needed)

```bash
chmod +x AgentPulse_0.1.0_amd64.AppImage
./AgentPulse_0.1.0_amd64.AppImage
```

### Linux — .deb (Ubuntu / Debian)

```bash
sudo dpkg -i AgentPulse_0.1.0_amd64.deb
agent-pulse
```

### Linux — .rpm (Fedora / RHEL)

```bash
sudo rpm -i AgentPulse-0.1.0-1.x86_64.rpm
agent-pulse
```

### Windows / macOS

Download the installer from [Releases](../../releases/latest).

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)
  ```bash
  cargo install tauri-cli
  ```
- **Linux** additional dependencies:
  ```bash
  # Ubuntu / Debian
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
  ```

### Build

```bash
git clone https://github.com/yazelin/AgentPulse.git
cd AgentPulse
cargo tauri build
```

Output binaries:

```
src-tauri/target/release/claude-pulse          # binary
src-tauri/target/release/bundle/deb/           # .deb package
src-tauri/target/release/bundle/rpm/           # .rpm package
src-tauri/target/release/bundle/appimage/      # .AppImage
```

### Development

```bash
cargo tauri dev
```

Since the frontend is static HTML/CSS/JS (no bundler), changes to `src/` files take effect on app restart.

## Usage

### First Launch

1. AgentPulse appears as a floating capsule at the top of your screen
2. A prompt asks to configure Claude Code hooks — click **Configure** to auto-setup `~/.claude/settings.json`
3. Start using Claude Code — the capsule updates in real-time

### Controls

| Action | Effect |
|--------|--------|
| **Hover** capsule | Expand to show all sessions |
| **Move mouse away** | Collapse back to capsule |
| **Drag** capsule | Reposition anywhere on screen |
| **Click** a session | Focus its terminal window |
| **Pin** button | Keep panel expanded without hovering |
| **Gear** button | Open settings |
| **System tray** → Show/Hide | Toggle visibility |
| **System tray** → Quit | Exit AgentPulse |

### Settings

- **Keep Expanded** — Pin the panel open without hovering
- **Sound on Complete** — Play a notification sound when AI finishes working (Glass, Ping, Pop, Chime, Bell)
- **Accent Color** — Purple, Cyan, Green, Orange, Pink
- **Size** — S / M / L text scaling

## How It Works

```
Claude Code  ──curl──▶  AgentPulse HTTP Server  ──▶  Capsule UI
(hooks)                 (localhost:19280)              (Tauri window)
```

1. AgentPulse starts a local HTTP server on port 19280-19289
2. On first launch, it configures hooks in `~/.claude/settings.json`
3. Claude Code sends events via `curl` on each action (session start, prompt submit, tool use, permission request, stop)
4. The capsule UI updates in real-time

### Hook Events

| Event | Triggers |
|-------|----------|
| `SessionStart` | New Claude Code session begins |
| `SessionEnd` | Session closes |
| `UserPromptSubmit` | User sends a prompt |
| `PreToolUse` / `PostToolUse` | Tool execution |
| `PermissionRequest` | Waiting for user approval |
| `Stop` | AI finishes working |

## Tech Stack

- **Tauri v2** — Cross-platform desktop framework
- **Rust** — Backend (HTTP server, session management, window control)
- **HTML / CSS / JS** — Frontend UI (no framework, no bundler)
- **Tokio** — Async runtime for the hook server
- **WebKitGTK** — Linux webview (via Tauri/WRY)

## Project Structure

```
AgentPulse/
├── src/                        # Frontend
│   ├── index.html              # Main HTML
│   ├── styles.css              # All styles
│   └── main.js                 # All logic (Tauri IPC, state, UI)
├── src-tauri/                  # Backend
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri config (window, tray, bundle)
│   ├── capabilities/           # Tauri v2 permissions
│   └── src/
│       ├── lib.rs              # App setup, tray, window management
│       ├── hook_server.rs      # HTTP server (tokio TCP)
│       ├── hook_event.rs       # Event data model
│       ├── session.rs          # Session state machine & manager
│       └── hooks_configurator.rs  # Auto-setup ~/.claude/settings.json
├── package.json
├── README.md
└── LICENSE
```

## Roadmap: Multi-CLI Support

AgentPulse is designed to grow beyond Claude Code to support multiple AI coding assistants:

| CLI | Status |
|-----|--------|
| Claude Code | Supported |
| Gemini CLI | Planned |
| GitHub Copilot CLI | Planned |
| OpenAI Codex CLI | Planned |

### Planned Design

- Each session row shows the corresponding **CLI icon** (instead of a colored dot)
- Capsule shows **active CLI icons** separated by `|` when multiple CLIs are in use
- **Config file** (`~/.config/agentpulse/config.json`) for per-CLI hook configuration

```
Capsule:  [Claude|Gemini]  my-app  Working...  00:14

Expanded:
  [Claude icon]   my-app       working     00:14
                  ~/projects/my-app
                  Fix the login bug
  [Gemini icon]   web-scraper  idle
                  ~/projects/web-scraper
  [Copilot icon]  api-server   waiting     01:23
                  ~/projects/api-server
                  Add rate limiting
```

## Known Limitations (Linux / X11)

| Issue | Workaround |
|-------|------------|
| Transparent `rgba()` backgrounds cause ghosting | Using opaque `rgb()` backgrounds with transparent window for rounded corners only |
| CSS `-webkit-app-region: drag` doesn't work | Using Tauri `startDragging()` API |
| `mouseleave` events unreliable on transparent windows | Using Tauri event system + CSS `:hover` polling |
| `<select>` dropdown uses system native styling | Replaced with custom div-based dropdown |

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms)

## License

MIT
