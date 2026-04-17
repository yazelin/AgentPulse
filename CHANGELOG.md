# Changelog

All notable changes land here. Each tagged release on
[GitHub Releases](https://github.com/yazelin/AgentPulse/releases) pulls its
description from the matching `## [vX.Y.Z]` section via `release.yml`.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.3] — 2026-04-17

Fixes spurious mid-turn completion sounds and swallowed end-of-turn
signals on Gemini CLI and GitHub Copilot.

### Fixed

- **Gemini: completion sound firing mid-turn** — `AfterModel` was mapped
  to `Stop`, but Gemini's `BeforeModel`/`AfterModel` pair fires once per
  model call, and a single turn typically runs 2+ model calls when tools
  are used. The mapping caused a `Completed` transition after every
  model call (playing the task-completed sound mid-turn), and then the
  real `AfterAgent` at end-of-turn was swallowed because state was
  already `Idle`. `AfterModel` now falls through to the state machine's
  no-op arm; `AfterAgent` alone carries the per-turn completion signal.
- **Copilot: `subagentStop` spuriously emitting completion** — same shape
  as the Gemini bug. A user prompt that spawns a subagent would fire
  `subagentStop` when the child finished (mid-turn), flipping state to
  `Idle` and firing the completion sound; the parent's `agentStop` would
  then be swallowed. `subagentStop` no longer maps to `Stop` — only
  `agentStop` does.

[v0.2.3]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.3

## [v0.2.2] — 2026-04-17

Hotfix for Gemini CLI hook execution on Windows.

### Fixed

- **Gemini CLI hooks on Windows** — Gemini hardcodes
  `powershell.exe -NoProfile -Command` for hook execution, and PowerShell
  parses `"path\to\exe.exe" arg` as a bare string expression (ParserError:
  UnexpectedToken at the argument) rather than a call. The Gemini hook
  command is now prefixed with PowerShell's `&` call operator on Windows
  only; cmd.exe and bash on other platforms still see the unchanged form.
- **Hook dedup marker** — `provider_needs_setup` / `remove_provider` looked
  for the substring `"agentpulse"` to identify previously installed
  AgentPulse hooks, but the v0.2 sidecar command contains
  `agent-pulse-hook` (hyphenated). The marker never matched, so toggling a
  provider on/off accumulated duplicate hook entries. Marker now matches
  the sidecar filename across shells and OSes.

[v0.2.2]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.2

## [v0.2.1] — 2026-04-16

First public release. Multi-provider hook monitoring with a sidecar-binary
architecture that works across Linux, macOS (Apple Silicon + Intel), and
Windows.

### Supported platforms

Each tag produces four zip archives: `linux`, `macos-arm64`, `macos-x64`,
`windows`. The two macOS zips are cross-compiled on the same Apple-Silicon
runner (GitHub retired the standalone Intel runner).

### Added

- **Sidecar hook binary** — CLI hooks invoke a standalone executable
  (`agent-pulse-hook`) instead of an inline bash one-liner. The sidecar reads
  the event JSON from stdin, resolves the server port from
  `~/.agentpulse/port`, and POSTs to the AgentPulse HTTP server. One command
  string works identically across bash, PowerShell, and cmd.exe, so
  Claude Code, Gemini CLI, Codex, and GitHub Copilot all share the same
  hook invocation.
- **Per-provider waiting sounds** — new `SessionTransition::StartedWaiting`
  fires a `task-waiting` event whenever a session first hits
  `WaitingForUser`; the frontend plays a dedicated per-provider clip
  independent of the completion clip. Ships eight bundled TTS sounds
  (`{provider}.mp3` + `{provider}-waiting.mp3`, voiced with
  `zh-TW-HsiaoChenNeural` / 曉臻).
- **Single-instance plugin** — `tauri-plugin-single-instance` forwards a
  second launch to the running window (show + focus) and exits. Fixes the
  ghost tray icon that appeared when a duplicate launch's HTTP server
  refused the port but the UI still spawned.
- **Landing page + live demo** — `docs/` is a GitHub Pages site with a
  full interactive AgentPulse embedded in an iframe (powered by a mocked
  Tauri IPC shim so the real `src/main.js` boots without a backend).
  Download buttons auto-fill from the GitHub releases API.
- **CHANGELOG + auto-populated release notes** — this file. The release
  workflow extracts the section matching the tag and passes it to
  `softprops/action-gh-release` as the draft body.
- **Project docs** — `CLAUDE.md` committed, covering the v0.2 architecture
  for anyone cloning the repo.

### Changed

- **Providers default to disabled** — toggling a provider on in Settings
  now immediately installs its hooks. Claude previously defaulted to
  `enabled: true` but the install flow didn't fire at startup, leaving the
  checkbox "on" with no actual hook installed.
- **Notification Sounds toggle label** — renamed from "Sound on Complete"
  to "Notification Sounds" since it now gates both completion and waiting
  clips.
- **Capsule bounce animation** — the collapse-to-capsule bounce runs as a
  pure CSS `@keyframes` animation (`capsuleCollapseBounce`) instead of the
  Rust-side `bounce_window` shim. Transform-only + 260 ms keeps it on the
  GPU and avoids X11 ghosting on transparent windows.
- **`seed_default_sounds` runs on every launch** — idempotent write, so
  existing installs pick up newly bundled defaults automatically.
- **CI speedups** — `cargo-binstall` for the Tauri CLI (dropped Windows
  install from ~10 min to ~45 s). Upgraded `actions/checkout@v5` and
  `actions/upload-artifact@v6` to clear Node 20 deprecation warnings.

### Fixed

- **Windows DWM border** — `"shadow": false` in `tauri.conf.json` removes
  the halo/ghost border visible on Windows 10/11.
- **Capsule count `3/3`** — when every session was active, the `/`
  separator was dropped and the counts jammed together as `33`. Always
  renders the slash now when at least one session is active.

### Known limitations

- **No installer** (`.msi` / `.deb` / `.dmg`) yet — bundle packaging of the
  sidecar binary across installer formats is still unresolved. Ships as
  zip only; keep the two binaries in the same folder.
- **Codex CLI on Windows** — upstream disables hook execution. AgentPulse
  still writes the hook config, so when OpenAI re-enables it the
  integration lights up automatically.
- **No code signing** — macOS users need `xattr -cr` to bypass Gatekeeper;
  Windows users see SmartScreen's "More info → Run anyway". Apple
  Developer cert ($99/yr) is the real fix — intentionally deferred.

[v0.2.1]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.1
