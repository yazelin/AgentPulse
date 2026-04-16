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
| SessionEnd | `SessionEnd` | — | — | `sessionEnd` |
| UserPromptSubmit | `UserPromptSubmit` | `BeforeModel` | `UserPromptSubmit` | `userPromptSubmitted` |
| PreToolUse | `PreToolUse` | `BeforeTool` | `PreToolUse` | `preToolUse` |
| PostToolUse | `PostToolUse` | `AfterTool` | `PostToolUse` | `postToolUse` |
| Stop | `Stop` | `AfterAgent` / `AfterModel` | `Stop` | `agentStop` / `subagentStop` |
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
| Any session, 10 min no events | → Stale (dim gray) |
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

**Status indicator colors** (theme-adaptive via CSS variables):

| State | Dark theme | Light theme |
|-------|-----------|-------------|
| Working | `rgb(77,242,153)` (light green) | `rgb(20,140,80)` (dark green) |
| Waiting | `rgb(255,179,64)` (light orange) | `rgb(217,119,6)` (dark orange) |
| Idle | gray (text-dim) | gray (text-dim) |
| Stale | dim gray | dim gray |

## Sound System

External MP3/WAV/OGG files in `~/.config/agentpulse/sounds/`. Each provider can have its own completion sound.

### Setup

1. Open Settings → **Sounds** tab → enable **Sound on Complete**
2. **Per-Provider Sounds** section shows one dropdown per provider
3. Click 📁 to open the sounds folder
4. Drop your MP3/WAV/OGG files there
5. Files appear in the dropdowns (each dropdown rescans on click)
6. Click ▶ next to each provider to preview

### Auto-matching

If a sound file starts with the provider ID (e.g., `claude.mp3`, `gemini.wav`), AgentPulse auto-assigns it on first launch.

### Bundled defaults

The repo's `sounds/` directory contains 4 default TTS sounds (Chinese voice "曉臻"). On first launch they are copied into `~/.config/agentpulse/sounds/`.

### Generate your own TTS sounds (optional)

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
- **Provider Icons** — Each session shows provider's official icon (lobehub/icons)
- **Status Dot** — Inline colored indicator next to project name
- **3-Line Session Info** — Project name + status, working directory, last prompt (italic)
- **Remove Session** — X button appears on row hover, click to remove
- **Smart Re-render** — Timer updates in-place, only structural changes trigger full re-render
- **Per-Provider Sounds** — Each CLI plays its own sound on completion (Rust rodio)
- **Bounce Animation** — Window bounces when collapsing
- **Draggable** — Drag capsule anywhere
- **Light / Dark Theme** — Toggle in Settings or Tray
- **System Tray** — Show/Hide, Open Settings, Toggle Theme, Open Config, Restart, Quit
- **Settings (Tabbed)** — Providers / Sounds / Appearance tabs
- **Open Provider Config** — Button per provider opens its CLI config file
- **Open AgentPulse Config** — Tray menu opens `~/.config/agentpulse/config.json`
- **GitHub Link** — Action bar button opens repo in browser
- **Auto-Detection** — First launch detects installed CLIs via `which`
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
agent-pulse
```

### Linux — .rpm (Fedora / RHEL)

```bash
sudo rpm -i AgentPulse-0.1.0-1.x86_64.rpm
agent-pulse
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
src-tauri/target/release/agent-pulse               # binary
src-tauri/target/release/bundle/deb/               # .deb
src-tauri/target/release/bundle/rpm/               # .rpm
src-tauri/target/release/bundle/appimage/          # .AppImage
```

### Development Workflow

> **Important**: Frontend files (`src/*`) are **embedded into the binary** at build time. Changes to HTML/CSS/JS require either rebuild OR using `watch.sh` (which uses `devUrl` mode).

Three scripts for different workflows:

```bash
./watch.sh         # DEV MODE — frontend hot-reloadable, Ctrl+R in window to refresh
./dev.sh           # Build debug binary + run (any change requires this)
./dev.sh release   # Build release binary + run
./reload.sh        # Just restart existing binary (no rebuild) — for testing startup flow
```

| Script | Frontend changes | Rust changes |
|--------|-----------------|-------------|
| `./watch.sh` | Ctrl+R refresh | Auto-rebuild |
| `./dev.sh` | Need rebuild | Need rebuild |
| `./reload.sh` | No effect (cached in old binary) | No effect |

`watch.sh` runs `cargo tauri dev` which serves frontend from `http://localhost:1420` via `npx serve`.

## Usage

### First Launch

1. AgentPulse opens with Settings showing detected providers (auto-checked if found via `which`)
2. Confirm/adjust provider selection — hooks are auto-installed/removed on toggle
3. (Optional) Switch to Sounds tab, enable sounds, customize per-provider
4. Close settings — the capsule is ready

### Controls

| Action | Effect |
|--------|--------|
| **Hover** capsule | Expand session list |
| **Move mouse away** | Collapse (with bounce animation) |
| **Drag** capsule | Reposition anywhere on screen |
| **Click** session row | Set as active session (highlight) |
| **Hover** session row | Show remove (X) button |
| **Click** X button | Remove session from list |
| **Pin** button | Keep panel expanded without hovering |
| **Gear** button | Open settings |
| **GitHub** button | Open repo in browser |

### System Tray

| Item | Effect |
|------|--------|
| **Show/Hide** | Toggle visibility (positions at current monitor top-center) |
| **Open Settings** | Show window + open settings panel |
| **Toggle Light/Dark** | Cycle theme |
| **Open Config File** | Open `~/.config/agentpulse/config.json` in default editor |
| **Restart** | Spawn new instance and exit current |
| **Quit** | Exit AgentPulse |

### Settings (Tabbed)

**Providers tab:**
- Toggle each CLI on/off (auto installs/removes hooks)
- 📝 button per provider opens its CLI config file
- Auto-detection: shows "detected" if CLI binary or config dir found
- Disabled providers show "coming soon" if hook setup not implemented

**Sounds tab:**
- Toggle "Sound on Complete"
- Per-provider sound dropdown (rescans folder on click)
- ▶ preview button per row
- 📁 opens sounds folder

**Appearance tab:**
- Light Theme toggle
- Keep Expanded toggle
- Accent Color (Purple, Cyan, Green, Orange, Pink)
- Size (S / M / L)

## Architecture

### How It Works

```
Claude Code ──curl──▶
Gemini CLI  ──curl──▶  AgentPulse HTTP Server
Codex CLI   ──curl──▶  (localhost:19280-19289)
Copilot CLI ──curl──▶
                              │
                              ▼
                      Session Manager
                      (state machine, timers)
                              │
                              ▼
                      Capsule UI (1s polling)
```

1. AgentPulse starts a TCP server on port 19280-19289 (tries each in range)
2. Port written to `~/.agentpulse/port`
3. On provider enable, hooks written to each CLI's config file (auto-cleans existing AgentPulse hooks before re-installing)
4. Each CLI sends JSON events via `curl` to `/hook/{provider_id}`
5. Server parses URL `/hook/{id}` → identifies provider
6. Field name normalization handles different CLI JSON conventions
7. Event names normalized to common PascalCase set
8. Session manager updates state machine
9. UI polls state every 1 second; smart re-render only on structural changes
10. On Stop event: emits `task-completed` event with provider ID → JS plays per-provider sound

### Hook Installation Details

The same `curl` command is generated by `curl_cmd(provider_id, port)` and inserted into each CLI's hook config:

```bash
curl -sf -m 2 -X POST -H 'Content-Type: application/json' \
  -d "$(cat)" \
  http://localhost:$(cat ~/.agentpulse/port 2>/dev/null || echo PORT)/hook/PROVIDER_ID || true
```

**Claude Code** (`~/.claude/settings.json`):
```json
{
  "hooks": {
    "SessionStart": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "curl ... /hook/claude", "async": true }]
    }]
  }
}
```

**Gemini CLI** (`~/.gemini/settings.json`):
```json
{
  "hooks": {
    "BeforeAgent": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "curl ... /hook/gemini", "async": true }]
    }]
  }
}
```

**Codex CLI** (`~/.codex/hooks.json` + `~/.codex/config.toml`):
```json
{
  "hooks": {
    "SessionStart": [{
      "hooks": [{ "type": "command", "command": "curl ... /hook/codex" }]
    }]
  }
}
```
And in `config.toml`:
```toml
[features]
codex_hooks = true
```
(Codex hooks are behind a feature flag in the current beta.)

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
    "theme": "dark",
    "pin_expanded": false,
    "sound_enabled": true,
    "provider_sounds": {
      "claude": "claude.mp3",
      "gemini": "gemini.mp3",
      "codex": "__none__",
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

`provider_sounds` value `"__none__"` means user explicitly chose no sound for that provider.

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
| Linux extras | `webkit2gtk`, `gtk`, `gdk` crates for window management |

## Project Structure

```
AgentPulse/
├── src/                           # Frontend
│   ├── index.html                 # Capsule, expanded, settings views (tabbed)
│   ├── styles.css                 # All styles, theme-adaptive via CSS vars
│   └── main.js                    # Tauri IPC, state, UI, provider icons, sound playback
├── src-tauri/                     # Backend (Rust)
│   ├── Cargo.toml                 # Dependencies (tauri, rodio, webkit2gtk, etc.)
│   ├── tauri.conf.json            # Window, tray, bundle, devUrl config
│   ├── capabilities/default.json  # Tauri v2 permissions
│   └── src/
│       ├── lib.rs                 # App setup, tray, window mgmt, all Tauri commands
│       ├── config.rs              # Config R/W, provider detection, sounds dir, default seeding
│       ├── hook_server.rs         # TCP HTTP server, URL routing, event normalization
│       ├── hook_event.rs          # RawHookEvent (field aliases) → HookEvent (normalized)
│       ├── session.rs             # State machine, SessionManager, AppState
│       └── hooks_configurator.rs  # Per-provider hook install/remove (4 different formats)
├── sounds/                        # Bundled default TTS sounds (zh-TW HsiaoChen voice)
│   ├── claude.mp3
│   ├── gemini.mp3
│   ├── codex.mp3
│   └── copilot.mp3
├── watch.sh                       # Dev mode with frontend hot-reload (devUrl)
├── dev.sh                         # Build debug/release binary + run
├── reload.sh                      # Restart existing binary (no rebuild)
├── package.json
├── README.md
└── LICENSE
```

## Known Limitations (Linux / X11)

| Issue | Workaround |
|-------|------------|
| `rgba()` backgrounds ghost on transparent windows | Opaque `rgb()` backgrounds; transparent window only for rounded corners |
| CSS `-webkit-app-region: drag` doesn't work | Tauri `startDragging()` API via JS `mousedown` |
| `mouseleave` unreliable on transparent windows | Tauri `cursor-left` event from Rust polling thread |
| `<select>` dropdown uses system native styling | Custom div-based dropdown |
| CSS `transition` / `animation` causes pixel ghosting | Most transitions removed; bounce via Rust `set_position` thread |
| `transform: translateZ(0)` creates black compositing layers | Not used |
| DOM re-render destroys hover state | Smart re-render: structural changes only; timers update in-place |
| Browser CSP blocks blob URLs and local files | Audio plays via Rust `rodio` (no browser audio at all) |
| Click-to-focus terminal window | Not implemented — gnome-terminal-server architecture makes per-window PID lookup unreliable |

## Credits

- Original [ClaudePulse](https://github.com/tzangms/ClaudePulse) by [@tzangms](https://github.com/tzangms)
- Provider icons from [@lobehub/icons](https://github.com/lobehub/lobe-icons)
- Audio playback via [rodio](https://github.com/RustAudio/rodio)
- Default TTS sounds via [edge-tts](https://github.com/rany2/edge-tts)

## License

MIT
