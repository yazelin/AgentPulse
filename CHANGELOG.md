# Changelog

All notable changes land here. Each tagged release on
[GitHub Releases](https://github.com/yazelin/AgentPulse/releases) pulls its
description from the matching `## [vX.Y.Z]` section via `release.yml`.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.1] — 2026-04-16

### Added
- **Intel Mac builds** — `release.yml` now cross-compiles `x86_64-apple-darwin`
  alongside `aarch64-apple-darwin` on `macos-latest`. Every tag produces four
  zips: `linux` / `macos-arm64` / `macos-x64` / `windows`. The previously
  attempted `macos-13` runner was retired by GitHub and had to be replaced
  with an in-process cross-compile.
- **Automated release notes** — this CHANGELOG. Each release workflow run
  extracts the section for its tag and drops it into the draft release as
  the body.
- **Project docs** — `CLAUDE.md` committed to the repo. Covers the v0.2
  architecture (sidecar binary, single-instance plugin, etc.) so anyone
  cloning has full project context.

### Changed
- **Capsule bounce animation** — the collapse-to-capsule bounce now runs as a
  pure CSS `@keyframes` animation (`capsuleCollapseBounce`) instead of the
  Rust-side `bounce_window` window-position shim. Transform-only + 260 ms
  keeps it on the GPU and avoids X11 ghosting.
- **Landing page** — `docs/` is now a full GitHub Pages site with a live
  interactive AgentPulse embedded in an iframe (mocked Tauri IPC shim so
  the real `src/main.js` boots without a backend), OS-specific download
  buttons that auto-fill from the GitHub releases API, real provider SVG
  icons, and a gradient-wallpaper hero.
- **Install docs** — README's install section split per OS. macOS entry
  spells out Apple Silicon vs Intel zip choice plus the `xattr -cr`
  workaround for Gatekeeper blocking the unsigned binaries.

### Fixed
- **Capsule count `3/3`** — when every session was active, the `/` separator
  was dropped and the counts jammed together as `33`. Always renders the
  slash now when at least one session is active.

## [v0.2.0] — 2026-04-16

Initial real release. Multi-provider hook monitoring with a sidecar-binary
architecture that works across Linux, macOS, and Windows.

### Added
- **Per-provider waiting sounds** — new `SessionTransition::StartedWaiting`
  fires a `task-waiting` event whenever a session first hits
  `WaitingForUser`; frontend plays a dedicated per-provider clip. Ships four
  bundled TTS clips ("Claude 等待回應" etc., voiced with
  `zh-TW-HsiaoChenNeural`) alongside the existing completion clips.
- **Single instance** — `tauri-plugin-single-instance` forwards a second
  launch to the running window (show + focus) and exits. Fixes the ghost
  tray icon that appeared when the HTTP server refused the port but the UI
  still spawned.
- **CI speedups** — `cargo-binstall` for the Tauri CLI (dropped Windows
  install from ~10 min to ~45 s). Upgraded `actions/checkout@v5` and
  `actions/upload-artifact@v6` to clear Node 20 deprecation warnings.
- **Release workflow** — `release.yml` triggered by `v*` tags. Builds
  three platforms and attaches zipped binaries to a draft GitHub Release.

### Changed
- **Hook architecture** — CLI hooks now invoke a standalone sidecar binary
  (`agent-pulse-hook`) which reads stdin and POSTs to the AgentPulse HTTP
  server. Replaces the v0.1 bash one-liner (`curl -d "$(cat)"…`) that broke
  on Windows. The sidecar works identically across bash / PowerShell /
  cmd.exe, so Claude Code, Gemini CLI, Codex, and GitHub Copilot all share
  the same hook command string.
- **Providers default to disabled** — toggling a provider on in Settings now
  immediately installs its hooks. Claude previously defaulted to
  `enabled: true` but the install flow didn't fire at startup, leaving the
  checkbox "on" with no actual hook installed.
- **Notification Sounds toggle label** — renamed from "Sound on Complete"
  to "Notification Sounds" since it now gates both completion and waiting
  clips.
- **`seed_default_sounds` on every launch** — idempotent write, so existing
  installs pick up newly bundled defaults automatically.

### Fixed
- **Windows DWM border** — `"shadow": false` in `tauri.conf.json` removes
  the halo/ghost border visible on Windows 10/11. An earlier attempt using
  `DWMWA_NCRENDERING_POLICY = DISABLED` regressed into an XP-style frame
  and was reverted.

### Known limitations
- No installer (`.msi` / `.deb` / `.dmg`) yet — bundle packaging of the
  sidecar binary across installer formats is still unresolved.
- Codex CLI disables hook execution on Windows; AgentPulse still writes the
  hook config, so when upstream re-enables it the integration lights up
  automatically.
- Binaries aren't code-signed; macOS users need `xattr -cr`, Windows users
  click through SmartScreen's "More info → Run anyway".

[v0.2.1]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.1
[v0.2.0]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.0
