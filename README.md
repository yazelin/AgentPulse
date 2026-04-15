# ClaudePulse (Cross-Platform)

A cross-platform desktop app that brings **Dynamic Island-inspired** real-time monitoring to your AI coding assistant sessions.

> This is a cross-platform version of [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms), rewritten with [Tauri](https://tauri.app/) to support **Linux**, **Windows**, and **macOS**.
> The original is a beautiful macOS-native app built with Swift/SwiftUI.

## Features

- **Dynamic Island Style** -- A compact capsule UI floats above your screen, expanding on hover to show full session details
- **Real-time Session Tracking** -- Monitor multiple sessions simultaneously with working, waiting, or idle status
- **Fully Local** -- All data stays on localhost via hooks. Nothing leaves your machine
- **System Tray** -- Quick controls from the system tray: show/hide panel
- **Zero Configuration** -- Automatically sets up hooks on first launch
- **Cross-Platform** -- Works on Linux, Windows, and macOS
- **Click to Focus** -- Click a session to bring its terminal window to the foreground

## Session States

| State | Description |
|-------|-------------|
| **Working** | AI is processing |
| **Waiting** | Waiting for user input or approval |
| **Idle** | Session is idle |
| **Stale** | No activity for over 10 minutes |

## Settings

- **Keep Expanded** -- Pin the panel open without hovering
- **Sound on Complete** -- Play a notification sound when work finishes (Glass, Ping, Pop, Chime, Bell)
- **Accent Color** -- Purple, Cyan, Green, Orange, Pink
- **Text Size** -- S / M / L

## Install

Download the latest release for your platform from [Releases](../../releases/latest).

### Build from Source

Prerequisites:
- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) (18+)
- System dependencies for Tauri v2 (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

```bash
git clone https://github.com/yazelin/ClaudePulse.git
cd ClaudePulse
cargo tauri build
```

For development:

```bash
cargo tauri dev
```

## Tech Stack

- **Tauri v2** -- Cross-platform desktop framework
- **Rust** -- Backend (HTTP hook server, session management, window management)
- **HTML/CSS/JS** -- Frontend UI
- **Tokio** -- Async runtime for the hook server

## How It Works

1. ClaudePulse starts a local HTTP server (port 19280-19289)
2. It configures Claude Code hooks in `~/.claude/settings.json`
3. Claude Code sends events (session start, tool use, stop, etc.) via `curl`
4. The capsule UI updates in real-time to show current status

## Roadmap: Multi-CLI Support

ClaudePulse is designed to support multiple AI coding assistants beyond Claude Code:

### Planned CLI Support

| CLI | Status |
|-----|--------|
| Claude Code | Supported |
| Gemini CLI | Planned |
| GitHub Copilot CLI | Planned |
| OpenAI Codex CLI | Planned |

### Architecture

Each CLI will have:
- Its own **icon** displayed in session rows (replacing the colored dot)
- **Capsule icon** showing active CLI icons (e.g. `Claude | Gemini` when both are active)
- A **config file** (`~/.config/claudepulse/config.json`) for per-CLI hook configuration

```
Capsule:  [Claude|Gemini]  my-app  Working...  00:14
Expanded:
  [Claude icon]  my-app      working    00:14
                 ~/projects/my-app
                 Fix the login bug
  [Gemini icon]  web-scraper  idle
                 ~/projects/web-scraper
  [Copilot icon] api-server   waiting    01:23
                 ~/projects/api-server
                 Add rate limiting
```

## Known Limitations (Linux/X11)

- **Transparent windows**: X11 + WebKitGTK has rendering issues with `rgba()` backgrounds on transparent windows. The app uses opaque backgrounds with transparent window corners for clean rounded edges.
- **Window dragging**: Uses Tauri `startDragging()` API instead of CSS `-webkit-app-region: drag` which doesn't work on Linux X11.
- **Hover detection**: Uses Tauri event system + `:hover` CSS polling as fallback for reliable collapse detection.

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms) -- the macOS-native version that inspired this project

## License

MIT
